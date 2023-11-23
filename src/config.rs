use std::{env, fs};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::exit;

use jars::JarOptionBuilder;
use regex::Regex;
use serde;
use serde_derive::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use toml::Value;

use crate::world::{DeepDirectoryDriver, Hasher, World};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub minecraft_jar: String,
    pub output_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub ignore_cache: bool,
    pub force_render: bool,
    pub worlds: HashMap<String, String>,
    pub textures: HashMap<String, String>,
    pub renders: Vec<Render>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Render {
    pub world: String,
    pub title: String,
    pub mode: String,
    pub dimension: String,
    pub textures: String,
}

impl Config {
    pub fn new(filename: &str) -> String {
        let mut default_cache = env::current_dir().unwrap();
        default_cache.push("cache");

        let mut config = Config {
            minecraft_jar: "".to_string(),
            output_dir: PathBuf::new(),
            cache_dir: default_cache.clone(),
            ignore_cache: false,
            force_render: false,
            worlds: Default::default(),
            textures: Default::default(),
            renders: vec![],
        };

        let mut render_list = vec![];

        // read contents of file
        let contents = fs::read_to_string(filename).unwrap();

        // parse the config as the vastly superior toml file
        let configuration = contents.parse::<Value>().unwrap();

        // todo - check if output key exists and panic if not

        // loop through and values
        for table in configuration.as_table() {
            for (key, val) in table.iter() {
                match key.as_str() {
                    // root level vars
                    "minecraft_jar" => config.minecraft_jar = Config::validate_minecraft_jar(val.to_string()),
                    "output" => config.output_dir = Config::validate_directory(val),
                    "cache" => config.cache_dir = Config::validate_directory(val),
                    "ignore_cache" => config.ignore_cache = val.as_bool().unwrap(),
                    "force_render" => config.force_render = val.as_bool().unwrap(),

                    // list of variables
                    "worlds" => config.worlds = Config::parse_worlds(val),
                    "textures" => config.textures = Config::parse_textures(val),
                    "renders" => render_list.extend(Config::parse_renders(val)),
                    _ => {
                        println!("unknown/ignored configuration setting: {:?}={:?}", &key, &val);
                    }
                }
            }
        }

        // find default minecraft jar if none was provided
        if 0 == config.minecraft_jar.len() {
            config.minecraft_jar = Config::validate_minecraft_jar(World::default_jar_path());
        }

        // extract necessary minecraft jar data
        config.extract_minecraft_jar();

        // validate render list
        config.renders = Config::validate_renders(render_list, &config);

        // save validated config to cache
        config.save_config()
    }

    fn save_config(&self) -> String {
        let mut rusty_config = env::current_dir().unwrap();
        rusty_config.push("system");
        if !rusty_config.exists() || !rusty_config.is_dir() {
            match fs::create_dir_all(&rusty_config) {
                Ok(_) => {}
                Err(err) => { eprintln!("Error creating config dir: {err}") }
            }
        }

        let content = match toml::to_string(&self) {
            Ok(content) => { content }
            Err(err) => {
                eprintln!("Error preparing cache data: {err}");
                exit(115)
            }
        };

        let hash = World::hash_string(content.clone());
        let mut filename = hash.trim_matches('"').to_owned();
        filename.push_str(".toml");

        rusty_config.push(filename.clone());

        match fs::write(rusty_config.as_path(), content) {
            Ok(_) => {}
            Err(err) => { eprintln!("Error writing cache data: {err}") }
        }

        filename
    }

    pub fn load_config(filename: String) -> Config {
        let mut rusty_config = env::current_dir().unwrap();
        rusty_config.push("system");
        rusty_config.push(filename);
        if !rusty_config.exists() { exit(117) }
        if rusty_config.is_dir() { exit(118) }

        let content = match fs::read_to_string(rusty_config) {
            Ok(content) => { content }
            Err(_) => { exit(122) }
        };

        let rusty_config: Config = match toml::from_str(&content.as_str()) {
            Ok(rusty_config) => { rusty_config }
            Err(_) => { exit(127) }
        };

        rusty_config
    }

    fn extract_minecraft_jar(&self) {
        // create unique cache hash for jar file
        let mut sha1 = Sha1::new();
        sha1.update(String::from(&self.minecraft_jar));
        let result = sha1.finalize();
        let hash = format!("{result:x}");

        // prepare target cache directories
        let mut cache_path = self.cache_dir.clone();
        cache_path.push("jar");
        cache_path.push(hash);

        // skip extraction process if it exists and we're using the cache
        if cache_path.exists() && false == self.ignore_cache {
            return;
        } else {
            println!("refreshing cache");
        }

        // if you pronounce gif wrong you probably say regex wrong too
        let pattern = match Regex::new(
            r"^assets..?minecraft..?(blockstates|models).*([\w]+\.json)$"
        ) {
            Ok(regex) => regex,
            Err(err) => {
                eprintln!("Error compiling regex pattern: {err}");
                exit(123)
            }
        };

        // open the minecraft jar file
        let jar = match jars::jar(&self.minecraft_jar, JarOptionBuilder::default()) {
            Ok(result) => result,
            Err(err) => {
                eprintln!("Error opening Minecraft jar: {err}");
                exit(99)
            }
        };

        // save the jar files
        for (file, bytes) in &jar.files {
            // skip files we don't care about
            if !pattern.is_match(&file) { continue; }

            // create cache path variable
            let mut cache_file = PathBuf::from(cache_path.clone());
            for element in file.split("/") { cache_file.push(element) }

            // make sure any non-existing parent dirs exist
            let parent_dirs = cache_file.parent().unwrap();
            if !parent_dirs.exists() {
                match fs::create_dir_all(&parent_dirs) {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("Error creating cache directory {:?}: {err}", &parent_dirs);
                        exit(151);
                    }
                }
            }

            // save bytes to disk
            match fs::write(&cache_file.as_path(), &bytes) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error writing to file: {err}");
                    exit(82)
                }
            }
        }
    }

    fn validate_minecraft_jar(input: String) -> String {
        let input = input.as_str().trim_matches('"');
        let target_jar = PathBuf::from(input);

        // check if it exists
        if !target_jar.exists() {
            if input == &World::default_jar_path() {
                panic!("Default minecraft jar couldn't be found.");
            }
            eprintln!("Minecraft jar could not be found: {:?}", &input);
            println!("Attempting to use default jar");
            return Config::validate_minecraft_jar(World::default_jar_path());
        }

        // make sure it's not a directory
        if target_jar.is_dir() {
            if input == &World::default_jar_path() {
                panic!("Default minecraft jar couldn't be found.");
            }
            eprintln!("Minecraft jar is a directory: {:?}", &input);
            println!("Attempting to use default jar");
            return Config::validate_minecraft_jar(World::default_jar_path());
        }

        input.to_string()
    }

    fn validate_directory(input: &Value) -> PathBuf {
        // convert path into string
        let binding = input.to_string();
        let input = binding.as_str().trim_matches('"');
        let target_dir = PathBuf::from(input);

        // check if it exists
        if !target_dir.exists() || !target_dir.is_dir() {
            // it doesn't exist so we gotta create it
            match fs::create_dir_all(&target_dir) {
                Ok(_) => {} // do nothing
                Err(err) => {
                    // this is a full stop because without the output, we cannot...well, output
                    panic!("Error while creating directory ({:?}): {}", &target_dir, err)
                }
            }
        }

        target_dir
    }

    fn validate_renders(render_list: Vec<Render>, config: &Config) -> Vec<Render> {
        let mut validated_list: Vec<Render> = vec![];

        for mut render_conf in render_list {
            let mut valid_world = false;
            let mut valid_dimension = false;
            let mut valid_textures = false;

            // validate world path
            let world_path = Path::new(&render_conf.world);
            if config.worlds.contains_key(&render_conf.world) {
                // has a valid world key
                render_conf.world = config.worlds[&render_conf.world].clone();
                valid_world = true;
            } else if world_path.exists() && world_path.is_dir() {
                // has a valid world path
                valid_world = true;
            } else {
                eprintln!("Invalid world path: {:?}", &world_path);
            }

            // validate the dimension
            let mut dimension_path = PathBuf::from(&render_conf.world);
            let dimension = match render_conf.dimension.to_lowercase().as_str() {
                "overworld" => { "" }
                "nether" => { "DIM1" }
                "the end" => { "DIM-1" }
                &_ => { &render_conf.dimension }
            };
            let mut subdirs = vec![dimension, "region"];
            subdirs.retain(|&x| 0 < x.len());
            for dir in subdirs { dimension_path.push(dir) }
            println!("dimension_path: {:?}", &dimension_path);
            if dimension_path.exists() && dimension_path.is_dir() {
                let binding = dimension_path.as_os_str().to_str().unwrap();
                render_conf.dimension = String::from(dimension_path.as_os_str().to_str().unwrap());
                valid_dimension = true;
            }
            println!("render_conf.dimension [{:?}]: {:?}", &valid_dimension, &render_conf.dimension);

            // validate textures path
            let textures_path = Path::new(&render_conf.textures);
            if config.textures.contains_key(&render_conf.textures) {
                render_conf.textures = config.textures[&render_conf.textures].clone();
                valid_textures = true;
            } else if textures_path.exists() && !textures_path.is_dir() {
                valid_textures = true;
            } else {
                println!("couldn't find textures '{:?}', using default", &render_conf.textures);
                render_conf.textures = config.textures["default"].clone();
                valid_textures = true;
            }

            // it's valid!
            let valid_checks = vec![valid_world, valid_dimension, valid_textures];
            if valid_checks.iter().all(|&x| x == true) {
                validated_list.push(render_conf);
            }
        }

        validated_list
    }

    fn all_true(all: Vec<bool>) -> bool {
        for a in all { if !a { return false; } }
        return true;
    }

    fn validate_dimension(input: &str) -> bool {
        println!("input: {:?}", &input);
        true
    }

    fn parse_worlds_old(input: &Value) -> HashMap<String, String> {
        let mut output: HashMap<String, String> = Default::default();

        match input.as_array() {
            Some(worlds) => {
                for world in worlds {
                    for data in world.as_table() {
                        // initialize
                        let mut id = String::new();
                        let mut path = String::new();

                        for (key, value) in data {
                            // extract values
                            match key.as_str() {
                                "id" => id = String::from(value.to_string().as_str().trim_matches('"')),
                                "path" => path = String::from(value.to_string().as_str().trim_matches('"')),
                                _ => {} // ignore
                            }
                        }

                        // validate and insert world
                        if Path::new(&path).is_dir() {
                            output.insert(id, path);
                        }
                    }
                }
            }
            None => {} // nothing exists
        }

        output
    }

    fn parse_worlds(input: &Value) -> HashMap<String, String> {
        let mut output: HashMap<String, String> = Default::default();

        match input.as_table() {
            Some(table_input) => {
                for (key, value) in table_input.iter() {
                    output.insert(String::from(key), String::from(value.as_str().unwrap()));
                }
            }
            None => {} // do nothing
        }

        output
    }

    fn parse_textures(input: &Value) -> HashMap<String, String> {
        println!("input: {:?}", &input);

        let mut output: HashMap<String, String> = Default::default();

        match input.as_table() {
            Some(table_input) => {
                for (key, value) in table_input.iter() {
                    output.insert(String::from(key), String::from(value.as_str().unwrap()));
                }
            }
            None => {} // do nothing
        }

        // add default texture path if missing
        if !output.contains_key("default") {
            output.insert(String::from("default"), World::default_jar_path());
        }

        output
    }

    fn parse_renders(input: &Value) -> Vec<Render> {
        let mut output = vec![];
        match input.as_array() {
            Some(renders) => {
                for render_settings in renders {
                    let render = Config::parse_render(render_settings);
                    // if 0 < render.
                    output.push(render);
                }
            }
            None => {}
        }
        output
    }

    fn parse_render(input: &Value) -> Render {
        let mut render = Render {
            world: "".to_string(),
            title: "My Render".to_string(),
            mode: "default".to_string(),
            dimension: "overworld".to_string(),
            textures: "default".to_string(),
        };

        match input.as_table() {
            Some(table) => {
                for (key, value) in table.iter() {
                    match key.as_str() {
                        "world" => render.world = String::from(value.to_string().as_str().trim_matches('"')),
                        "title" => render.title = String::from(value.to_string().as_str().trim_matches('"')),
                        "mode" => render.mode = String::from(value.to_string().as_str().trim_matches('"')),
                        "dimension" => render.dimension = String::from(value.to_string().as_str().trim_matches('"')),
                        "textures" => render.textures = String::from(value.to_string().as_str().trim_matches('"')),
                        _ => {} // ignore unknown keys
                    }
                }
            }
            None => {} // do nothing
        }

        render
    }
}
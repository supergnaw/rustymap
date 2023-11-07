use std::fs;
use std::collections::HashMap;
use std::path::Path;

use toml::Value;

use crate::world::{DeepDirectoryDriver, World};

#[derive(Debug)]
pub struct Config {
    pub output_dir: String,
    pub worlds: HashMap<String, String>,
    pub textures: HashMap<String, String>,
    pub renders: Vec<Render>,
}

#[derive(Debug)]
pub struct Render {
    pub world: String,
    pub title: String,
    pub mode: String,
    pub dimension: String,
    pub textures: String,
}

impl Config {
    pub fn load(filename: &str) -> Config {
        // initialize Config struct with defaults
        let mut config = Config {
            output_dir: "".to_string(),
            worlds: Default::default(),
            textures: Default::default(),
            renders: vec![],
        };

        let mut render_list = vec![];

        println!("\n\n");

        // read contents of file
        let contents = fs::read_to_string(filename).unwrap();

        // parse the config as the vastly superior toml file
        let configuration = contents.parse::<Value>().unwrap();

        // loop through and values
        for table in configuration.as_table() {
            // root table
            for (key, val) in table.iter() {
                match key.as_str() {
                    "output" => config.output_dir = Config::validate_output(val),
                    "worlds" => config.worlds = Config::parse_worlds(val),
                    "textures" => config.textures = Config::parse_textures(val),
                    "renders" => render_list.extend(Config::parse_renders(val)),
                    _ => {
                        println!("unknown/ignored configuration setting: {:?}={:?}", &key, &val);
                    }
                }
            }
        }

        // validate render list
        // config.renders = Config::validate_renders(render_list, &config.worlds);
        config.renders = Config::validate_renders_new(render_list, &config);

        config
    }

    fn validate_renders_new(render_list: Vec<Render>, config: &Config) -> Vec<Render> {
        let mut validated_list: Vec<Render> = vec![];

        for render_conf in render_list {
            // world key is missing
            if !config.worlds.contains_key(&render_conf.world) {
                println!("ignoring world {:?}: undefined world path", &render_conf.world);
                continue;
            }

            // texture key is missing
            if !config.textures.contains_key(&render_conf.textures) {
                print!("ignoring render: unknown textures: {:?}", &render_conf.textures);
                continue;
            }

            validated_list.push(render_conf);
        }

        validated_list
    }

    fn validate_renders(render_list: Vec<Render>, worlds: &HashMap<String, String>) -> Vec<Render> {
        let mut validated_list: Vec<Render> = vec![];

        for render_conf in render_list {
            // verify target world exists--no misspellings allowed!
            if !worlds.contains_key(&render_conf.world) { continue }

            // validate dimension here
            if !Config::validate_dimension(&worlds[&render_conf.world]) { continue }

            // validation checks passed, add to validated list for output
            validated_list.push(render_conf)
        }


        validated_list
    }

    fn validate_dimension(input: &str) -> bool {
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
                    println!("key={:?}, value{:?}", &key, &value);
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

    fn validate_output(input: &Value) -> String {
        // convert path into string
        let target_dir = String::from(input.to_string().as_str().trim_matches('"'));

        // check if it exists
        if !Path::new(&target_dir).exists() {
            // it doesn't exist so we gotta create it
            match fs::create_dir_all(&target_dir) {
                Ok(_) => {} // do nothing
                Err(err) => {
                    // this is a full stop because without the output, we cannot...well, output
                    panic!("Error while creating output directory ({:?}): {}", &target_dir, err)
                }
            }
        }

        // make sure if it exists that it's a directory
        else if !Path::new(&target_dir).is_dir() {
            // it exists but as a file, so lets make the actual directory
            match fs::create_dir_all(&target_dir) {
                Ok(_) => {} // do nothing
                Err(err) => {
                    // this is a full stop because without the output, we cannot...well, output
                    panic!("Error while creating output directory ({:?}): {}", &target_dir, err)
                }
            }
        }

        target_dir
    }
}
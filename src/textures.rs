use std::collections::HashMap;
use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;
use sha1;
use sha1::{Sha1, Digest};
use regex::Regex;
use jars::JarOptionBuilder;
use zip::read::ZipArchive;
use crate::world::{DeepDirectoryDriver, World};

pub struct TexturePack {
    pub filepath: String,
    pub block_table: HashMap<String, String>,
}

impl TexturePack {
    pub fn new() {
        let install_path = World::default_jar_path();
        println!("install_path: {:?}", install_path);
        exit(17);
    }

    pub fn load(textures: HashMap<String, String>) -> HashMap<String, PathBuf> {
        let mut output: HashMap<String, PathBuf> = Default::default();

        for (name, path) in textures {
            // get file hash of texture path for cache
            let hash = TexturePack::file_hash(&path);

            // create path buffer for cache directory
            let mut path_buf = env::current_dir().unwrap();
            for subdir in ["cache", "textures", &hash] { path_buf.push(subdir) }

            // add to output
            output.insert(name, path_buf.clone());

            // skip if it exists
            if path_buf.exists() {
                continue
            }

            // create it if not exists
            match fs::create_dir_all(&path_buf) {
                Ok(_) => {}, // no news is good news
                Err(err) => eprintln!("Error creating cache: {}", err),
            }

            // extract texture files
            match &path[path.len() - 3..] {
                "jar" => TexturePack::extract_jar_textures(&path, &path_buf),
                "zip" => TexturePack::extract_zip_textures(&path, &path_buf),
                _ => println!("unsupported texture container: {:?}", &path),
            }
        }

        output
    }

    fn file_hash(file_path: &str) -> String {
        let mut file = File::open(file_path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        let mut sha1 = Sha1::new();
        sha1.update(&buffer);

        let result = sha1.finalize();

        format!("{result:x}")
    }

    fn extract_jar_textures(path: &String, path_buf: &PathBuf) {
        // file pattern
        let pattern = Regex::new(r"^assets.minecraft.textures.(.+\.png)$").expect("the unexpected");

        let mut bytes_written = 0;

        let jar = match jars::jar(&path, JarOptionBuilder::default()) {
            Ok(result) => {
                println!("Successfully opened texture jar");
                result
            }
            Err(e) => {
                println!("Error opening texture jar: {e}");
                exit(48)
            }
        };

        for (file, bytes) in &jar.files {
            if pattern.is_match(&file) {
                // create cache path variable
                let mut cache_path = std::path::PathBuf::from(&path_buf);
                let subdirs: Vec<&str> = pattern.captures(&file).unwrap()
                    .get(1).unwrap().as_str().split("/").collect();
                for subdir in subdirs { cache_path.push(subdir) }

                // skip if it exists
                if cache_path.exists() { continue }

                // create any non-existing subdirectories in the cache
                if !cache_path.parent().unwrap().exists() {
                    match fs::create_dir_all(&cache_path.parent().unwrap()) {
                        Ok(_) => {}, // no need to log this
                        Err(err) => eprintln!("Error creating cache: {err}"),
                    }
                }

                // write the bytes to the file
                match fs::write(&cache_path.as_path(), &bytes) {
                    Ok(_) => bytes_written += &bytes.len(),
                    Err(err) => {
                        eprintln!("Error writing to file: {err}");
                        exit(82)
                    },
                }
            }
        }

        // good job, team! we did it!
        println!("{:?} bytes successfully written", &bytes_written);
    }

    fn extract_zip_textures(path: &String, path_buf: &PathBuf) {
        // file pattern
        let pattern = Regex::new(r"^assets.minecraft.textures.(.+\.png)$").expect("the unexpected");

        let mut bytes_written = 0;

        // open raw file
        let file = match File::open(&path) {
            Ok(result) => {
                println!("Successfully opened zip archive");
                result
            }
            Err(err) => {
                println!("Error opening zip archive: {err}");
                exit(93)
            }
        };

        // handle archive zontents
        let mut archive = match ZipArchive::new(file) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Error reading ZipFile: {e}");
                exit(105);
            }
        };

        // find target files
        for i in 0..archive.len() {
            // get ZipFile from archive by index
            let mut zipfile = match archive.by_index(i) {
                Ok(result) => result,
                Err(err) => {
                    eprintln!("Error selecting ZipFile from archive: {err}");
                    exit(124);
                }
            };

            // use regex pattern to see if this is a file we want
            let filename = String::from(zipfile.name());
            if !pattern.is_match(&filename) {
                // skip this file
                continue
            }

            // build full path string from current path buffer
            let mut zip_path = std::path::PathBuf::from(&path_buf);
            let subdirs: Vec<&str> = pattern.captures(&filename).unwrap()
                .get(1).unwrap().as_str().split("/").collect();
            for subdir in subdirs { zip_path.push(subdir) }

            // create parent folders if not in existance
            if !zip_path.parent().unwrap().exists() {
                match fs::create_dir_all(&zip_path.parent().unwrap()) {
                    Ok(_) => {}, // less logging = more speed
                    Err(err) => {
                        eprintln!("Error creating directories: {err}");
                        exit(118)
                    },
                }
            }

            // read the file bytes
            let mut bytes: Vec<u8> = vec![];
            zipfile.read_to_end(&mut bytes);

            // create target file handle to write to
            let mut target_file = match File::create(&zip_path) {
                Ok(result) => { result },
                Err(err) => {
                    eprintln!(
                        "Error creating target file to copy from zip archive ({:?}): {err}"
                        , &zip_path
                    );
                    exit(153)
                }
            };

            // write the file
            match target_file.write_all(&bytes) {
                Ok(_) => bytes_written += &bytes.len(),
                Err(err) => {
                    eprintln!("Error writing to file: {}", err);
                    exit(82)
                },
            }
        }

        // good job, team! we did it!
        println!("{:?} bytes successfully written", &bytes_written);
    }
}
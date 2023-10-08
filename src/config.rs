use std::{fs, process::exit};
use toml::de;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub world_dir: String,
    pub output_dir: String,
    pub texture_path: String,
}

pub fn load(filename: &str) -> Config {
    let contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Could not read file `{}`: {:?}", filename, err);
            exit(1);
        }
    };
    let config: Config = match de::from_str(&contents) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Unable to load data from `{}`: {:?}", filename, error);
            exit(1);
        }
    };
    config
}
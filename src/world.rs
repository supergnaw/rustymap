use std::{
    fs,
    path::{
        PathBuf,
    },
};
use std::fs::DirEntry;
use std::process::exit;
use regex::Regex;
use crate::region::*;


#[derive(Debug)]
pub struct World {
    pub world_path: String,
    pub regions: Vec<Region>,
    pub poi: Vec<u8>,
    pub players: Vec<u8>,
    pub entities: Vec<u8>,
    pub level: Vec<u8>,
}

impl World {
    pub fn new(world_path: &str) -> World {
        println!("collecting world data from: {:?}", &world_path);

        let mut world = World {
            world_path: String::from(world_path),
            level: vec![],
            regions: vec![],
            entities: vec![],
            players: vec![],
            poi: vec![],
        };
        world.load_regions();

        println!("successfully loaded world data.");

        world
    }

    pub fn load_level(&mut self) {
        todo!()
    }

    pub fn load_regions(&mut self) {
        let mut region_path = PathBuf::from(&self.world_path);
        let _ = region_path.push("region");
        if !region_path.exists() || !region_path.is_dir() {
            return;
        }

        let mut region_files = vec![];
        match fs::read_dir(region_path) {
            Ok(results) => {
                for result in results {
                    region_files.push(result);
                }
            },
            Err(err) => {
                println!("Error reading region path: {:?}", err);
                exit(42069)
            }
        }

        let region_file_count = region_files.len();
        let mut loading_count = 0;

        for region_file in region_files {
            // counter
            loading_count += 1;
            println!("loading region {:?}/{:?}:", loading_count, region_file_count);

            // load the region
            let dir_entry = *&region_file.as_ref().unwrap();
            let file_path = dir_entry.path().to_string_lossy().to_string();
            if file_path.ends_with(".mca") || file_path.ends_with(".mcr") {
                let region = Region::new(&file_path);
                let _ = self.regions.push(region);
            }
        }
    }

    pub fn load_entities(&mut self) {
        todo!()
    }

    pub fn load_players(&mut self) {
        todo!()
    }

    pub fn load_poi(&mut self) {
        todo!()
    }
}


pub trait DeepDirectoryDriver {
    fn default_jar_path() -> String;
}

impl DeepDirectoryDriver for World {
    fn default_jar_path() -> String {
        // get default home directory
        let mut install_path= dirs::home_dir().expect("Invalid home directory");

        // get windows-specific install path
        if cfg!(target_os = "windows") {
            for subdir in ["AppData", "Roaming", ".minecraft", "versions"] {
                install_path.push(subdir);
            }
        }

        // get linux-specific install path
        if cfg!(target_os = "linux") {
            for subdir in [".minecraft", "versions"] {
                install_path.push(subdir);
            }
        }

        // get macos-specific install path
        if cfg!(target_os = "macos") {
            for subdir in ["Library", "Application Support", "minecraft", "versions"] {
                install_path.push(subdir);
            }
        }

        // read directory for installed versions
        let entries = match fs::read_dir(&install_path) {
            Ok(results) => { results }
            Err(err) => {
                println!("Error reading directory: {err}\n{:?}", &install_path);
                exit(38);
            }
        };

        let pattern = Regex::new(r"^\d+\.\d+(\.\d)?$").expect("the unexpected");
        let mut newest = String::from("0.0.0");

        for entry in entries {
            // filter result
            let entry: DirEntry = entry.expect("the unexpected");

            /*
             how do I check if it's a file or directory?
             */

            // get filename
            let filename = String::from(entry.file_name().to_str().unwrap());

            // exclude not-applicable subdirs via fancy pancy rejular expression
            newest = match pattern.is_match(&filename) {
                // weird string trickery to do magic with numbers
                true => { World::newer_version(&newest, &filename) }
                false => { String::from(&newest) }
            };
        }

        install_path.push(&newest);
        newest.push_str(".jar");
        install_path.push(&newest);

        String::from(install_path.to_str().unwrap())
    }
}

pub trait Versioning {
    fn newer_version(ver_1: &str, ver_2: &str) -> String;
    fn version_int(version_number: &str) -> usize;
}

impl Versioning for World {
    fn newer_version(ver_1: &str, ver_2: &str) -> String {
        let val_1 = World::version_int(&ver_1);
        let val_2 = World::version_int(&ver_2);

        match val_1 < val_2 {
            true => String::from(ver_2),
            false => String::from(ver_1)
        }
    }

    fn version_int(version_number: &str) -> usize {
        let parts: Vec<&str> = version_number.split(".").collect();
        let mut output = 0;

        match parts.clone().into_iter().count() {
            3 => {
                // minor releases, A.B.C
                output += parts[0].parse::<usize>().unwrap() << 16;
                output += parts[1].parse::<usize>().unwrap() << 8;
                output += parts[2].parse::<usize>().unwrap();
            }
            2 => {
                // major releases, A.B
                output += parts[0].parse::<usize>().unwrap() << 16;
                output += parts[1].parse::<usize>().unwrap() << 8;
            }
            _ => {
                // invalid
            }
        }

        output
    }
}
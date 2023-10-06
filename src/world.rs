use std::{
    fs,
    path::{
        PathBuf,
    },
};
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

pub trait WorldLoader {
    fn new(world_path: &str) -> World;


    fn load_level(&mut self);


    fn load_regions(&mut self);


    fn load_entities(&mut self);


    fn load_players(&mut self);


    fn load_poi(&mut self);
}

impl WorldLoader for World {
    fn new(world_path: &str) -> World {
        let mut world = World {
            world_path: String::from(world_path),
            level: vec![],
            regions: vec![],
            entities: vec![],
            players: vec![],
            poi: vec![],
        };
        world.load_regions();
        world
    }


    fn load_level(&mut self) {
        todo!()
    }


    fn load_regions(&mut self) {
        // todo: remove region_files as the data is stored in `regions`
        let mut region_path = PathBuf::from(&self.world_path);
        let _ = region_path.push("region");
        if !region_path.exists() || !region_path.is_dir() {
            return;
        }

        for entry in fs::read_dir(region_path).unwrap() {
            let dir_entry = *&entry.as_ref().unwrap();
            let file_path = dir_entry.path().to_string_lossy().to_string();
            if file_path.ends_with(".mca") || file_path.ends_with(".mcr") {
                let region = Region::new(&file_path);
                let _ = self.regions.push(region);
            }
        }
    }


    fn load_entities(&mut self) {
        todo!()
    }


    fn load_players(&mut self) {
        todo!()
    }


    fn load_poi(&mut self) {
        todo!()
    }
}

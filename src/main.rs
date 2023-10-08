mod world;
mod level;
mod region;
mod chunk;
mod tag;
mod nbt;
mod config;
mod error;

use crate::world::*;

fn main() {
    // load config, currently hard-coded to be in the same directory as the exe
    let config = config::load("./config.toml");
    let _ = World::new(&config.world_dir);
    println!("Baby's first Minecraft parser finished successfully!")
}


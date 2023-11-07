mod world;
mod level;
mod region;
mod chunk;
mod tag;
mod nbt;
mod config;
mod error;
mod args;
mod textures;

use crate::world::*;
use crate::args::*;
use crate::config::Config;
use crate::textures::TexturePack;

fn main() {
    // parse command line arguments
    let args: Args = ArgParse::load();

    // load config file
    let config = Config::load(&args.config_file);
    dbg!(&config);

    // load textures
    let textures = TexturePack::load(config.textures.clone());
    dbg!(&textures);

    // collect world data
    // let world = World::new(&config.world_dir);

    // we are complete
    println!("Baby's first Minecraft parser finished successfully!")
}

fn texture_path_valid(target: &str) {
    let texture_path = String::from(target);
    dbg!(&texture_path);
}
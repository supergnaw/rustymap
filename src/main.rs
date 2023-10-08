mod world;
mod level;
mod region;
mod chunk;
mod tag;
mod nbt;
mod config;
mod error;
mod args;

use log::debug;
use crate::world::*;
use crate::args::*;

fn main() {
    // parse command line arguments
    let args: Args = ArgParse::load();

    // load config file
    let config = config::load(&args.config_file);
    dbg!(&config);

    // collect world data
    let world = World::new(&config.world_dir);
    println!("world.regions[0].chunks[0].nbt.tags: {:?}", &world.regions[0].chunks[0].nbt.tags);

    // we are complete
    println!("Baby's first Minecraft parser finished successfully!")
}


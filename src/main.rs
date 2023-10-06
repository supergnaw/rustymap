use std::{
    io::{
        Read,
        prelude::*,
    }
};

mod world;

use crate::world::*;

// mod level;
// use crate::level::*;
mod region;

use crate::region::*;

mod chunk;

use crate::chunk::*;

mod tag;

use crate::tag::*;

mod nbt;

use crate::nbt::*;

mod error;
mod config;

const DEBUG: bool = true;

fn main() {
    // load config, currently hard-coded to be in the same directory as the exe
    let config = config::load("./config.toml");
    let _ = World::new(&config.world_dir);
}


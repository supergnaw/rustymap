//! CHUNK
//!
//! Chunk payload data begins with a (big-endian) four-byte signed length field that indicates
//! the exact length of the remaining chunk data in bytes. The following byte indicates the
//! compression scheme used for chunk data, and the remaining (length-1) bytes are the compressed
//! chunk data.
//!
//! | BYTES -->       | 0  | 1  | 2  | 3  |         4        |                 5                |
//! |-----------------|-------------------|------------------|----------------------------------|
//! | DESCRIPTION --> | length (in bytes) | compression type | compressed data (length-1 bytes) |

//! CHUNK BLOCKS
//!
//! Chunk block data stored as 16x16x16 block sections in the chunk payload. These sections are
//! identified in the tag root::sections[]block_states::data. The actual block data is stored as an
//! array of 64-bit integers totalling 4096 indices. All indices are a minimum of 4-bits in length
//! and are all the same length: equal to the minimum amount of bits required to identify the
//! highest pallet index. If the bits of an index were to span from the end of one 64-bit integer to
//! the beginning of the next 64-bit integer, it is shifted wholly into the next 64-bit integer,
//! starting on the same bit as the next integer, e.g. if each index were seven bits long, the ninth
//! index would start on the sixty-third bit of the 64-bit integer in the array and overlap into the
//! next integer, thus the next index is shifted "to the right" by one bit. If the chunk section
//! contains only a single block in the pallet, the data tag is completely omitted.

// use std::any::Any;
use std::io::Read;
use std::process::exit;
use std::usize;

use flate2::read::{GzDecoder, ZlibDecoder};

use crate::nbt::*;

#[derive(Debug)]
pub struct Chunk {
    nbt: NBT,
    pub data_version: i32,
    pub x_pos: i32,
    pub z_pos: i32,
    pub y_pos: i32,
    pub status: ChunkStatus,
    pub last_update: i32,
    pub sections: Vec<ChunkSection>,
    pub block_entities: Vec<BlockEntity>,
    pub carving_masks: CarvingMask,
    pub heightmaps: Heightmap,
    pub lights: Vec<Vec<Box<str>>>,
    pub fluid_ticks: Vec<TileTick>,
    pub block_ticks: Vec<TileTick>,
    pub inhabited_time: i64,
    pub structures: Vec<Structure>,
}

impl Chunk {
    pub fn new(bytes: Vec<u8>) -> Self {

        let raw_bytes = Chunk::decompress(bytes);

        let chunk = Chunk {
            nbt: NBT::new(&raw_bytes),
            data_version: 0,
            x_pos: 0,
            z_pos: 0,
            y_pos: 0,
            status: ChunkStatus::Empty,
            last_update: 0,
            sections: vec![],
            block_entities: vec![],
            carving_masks: CarvingMask::new(),
            heightmaps: Heightmap::new(),
            lights: vec![vec![]],
            fluid_ticks: vec![],
            block_ticks: vec![],
            inhabited_time: 0,
            structures: vec![],
        };

        chunk
    }

    fn decompress(bytes: Vec<u8>) -> Vec<u8> {
        // get chunk size
        let size_bytes: [u8; 4] = [bytes[0], bytes[1], bytes[2], bytes[3]];
        let size: usize = u32::from_be_bytes(size_bytes) as usize;

        // get compression type
        let compression_type: usize = bytes[4] as usize;

        // decompress bytes
        let raw_bytes: Vec<u8> = bytes[5..5 + size].to_vec();
        let mut decompressed: Vec<u8> = vec![];
        return match compression_type {
            1 => {
                let mut decoder = GzDecoder::new(&raw_bytes as &[u8]);
                match decoder.read_to_end(&mut decompressed) {
                    Ok(size) => { decompressed },
                    Err(err) => { println!("Error: {:?}", err); exit(42069); }
                }
            },
            2 => {
                let mut decoder = ZlibDecoder::new(&raw_bytes as &[u8]);
                match decoder.read_to_end(&mut decompressed) {
                    Ok(size) => { decompressed },
                    Err(err) => { println!("Error: {:?}", err); exit(42069); }
                }
            },
            _ => {
                bytes.to_vec()
            }
        }
    }
}


#[derive(Debug)]
pub struct ChunkRoot {
    data_version: i32,
    x_pos: i32,
    z_pos: i32,
    y_pos: i32,
    status: ChunkStatus,
    last_update: i32,
    sections: Vec<ChunkSection>,
    block_entities: Vec<BlockEntity>,
    carving_masks: CarvingMask,
    heightmaps: Heightmap,
    lights: Vec<Vec<Box<str>>>,
    fluid_ticks: Vec<TileTick>,
    block_ticks: Vec<TileTick>,
    inhabited_time: i64,
    structures: Vec<Structure>,
}

impl ChunkRoot {
    pub fn new() -> Self {
        ChunkRoot {
            data_version: 0,
            x_pos: 0,
            z_pos: 0,
            y_pos: 0,
            status: ChunkStatus::Empty,
            last_update: 0,
            sections: vec![],
            block_entities: vec![],
            carving_masks: CarvingMask::new(),
            heightmaps: Heightmap::new(),
            lights: vec![vec![]],
            fluid_ticks: vec![],
            block_ticks: vec![],
            inhabited_time: 0,
            structures: vec![],
        }
    }
}

#[derive(Debug)]
pub struct ChunkSection {
    y: i32,
    block_states: ChunkBlockStates,
    biomes: ChunkBiomes,
    block_light: [u8; 4096],
    sky_light: [u8; 4096],
}

#[derive(Debug)]
pub struct BlockEntity {
    entity_type: BlockEntityType,
    value: Box<str>,
}

#[derive(Debug)]
pub struct CarvingMask {
    air: Vec<u8>,
    liquid: Vec<u8>,
}

impl CarvingMask {
    fn new() -> Self {
        CarvingMask {
            air: vec![],
            liquid: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Heightmap {
    motion_blocking: [u16; 256],
    motion_blocking_no_leaves: [u16; 256],
    ocean_floor: [u16; 256],
    ocean_floor_wg: [u16; 256],
    world_surface: [u16; 256],
    world_surface_wg: [u16; 256],
}

impl Heightmap {
    pub fn new() -> Self {
        Heightmap {
            motion_blocking: [0u16; 256],
            motion_blocking_no_leaves: [0u16; 256],
            ocean_floor: [0u16; 256],
            ocean_floor_wg: [0u16; 256],
            world_surface: [0u16; 256],
            world_surface_wg: [0u16; 256],
        }
    }
}

#[derive(Debug)]
pub struct TileTick {
    i: Box<str>,
    p: Vec<Box<str>>,
    t: i32,
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug)]
pub struct Structure {
    structure_name: Box<str>,
    x: i32,
    z: i32,
}

#[derive(Debug)]
pub struct ChunkBlockStates {
    palette: Vec<ChunkBlockState>,
    data: Vec<u64>,
}

#[derive(Debug)]
pub struct ChunkBiomes {
    pallet: Vec<Box<str>>,
    data: Vec<u64>,
}

#[derive(Debug)]
pub struct ChunkBlockState {
    name: Box<str>,
    properties: Vec<Box<BlockState>>,
}

#[derive(Debug)]
pub struct BlockState {
    name: Box<str>,
    value: Box<str>,
}

#[derive(Debug)]
pub enum ChunkStatus {
    //! Currently a placeholder, but I do not think I'll even use this beyond detecting whether the
    //! chunk is a spawn chunk or if it has been completely generated and shows "minecraft:full".
    Empty,
    // StructureStarts,
    // StructureReferences,
    // Biomes,
    // Noise,
    // Surface,
    // Carvers,
    // LiquidCarvers,
    // Features,
    // Light,
    // Spawn,
    // Heightmaps,
    // MinecraftFull,
}

#[derive(Debug)]
pub enum BlockEntityType {
    //! Maybe look into incorporating these dynamically via a .toml file so as to support new blocks
    //! as they are released without requiring new builds, while also supporting expanded block
    //! palettes from mods? I dunno, just a thought...
    // // https://minecraft.wiki/w/Chunk_format#Block_entity_format
    // Banner,
    // Barrel,
    // Beacon,
    // Bed,
    // Beehive,
    // Bell,
    // BlastFurnace,
    // BrewingStand,
    // Campfire,
    // ChiseledBookshelf,
    // Chest,
    // Comparator,
    // CommandBlock,
    // Conduit,
    // DaylightDetector,
    // Dispenser,
    // Dropper,
    // EnchantingTable,
    // EnderChest,
    // EndGateway,
    // EndPortal,
    // Furnace,
    // Hopper,
    // Jigsaw,
    // Jukebox,
    // Lectern,
    // MobSpawner,
    // Piston,
    // ShulkerBox,
    // Sign,
    // Skull,
    // Smoker,
    // SoulCampfire,
    // StructureBlock,
    // TrappedChest,
}

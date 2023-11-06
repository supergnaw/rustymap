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
//! highest palette index. If the bits of an index were to span from the end of one 64-bit integer to
//! the beginning of the next 64-bit integer, it is shifted wholly into the next 64-bit integer,
//! starting on the same bit as the next integer, e.g. if each index were seven bits long, the ninth
//! index would start on the sixty-third bit of the 64-bit integer in the array and overlap into the
//! next integer, thus the next index is shifted "to the right" by one bit. If the chunk section
//! contains only a single block in the palette, the data tag is completely omitted.

use std::usize;
use std::cmp::max;
use std::collections::HashMap;
use std::io::Read;
use std::process::exit;

use flate2::read::{GzDecoder, ZlibDecoder};

use crate::nbt::*;
use crate::tag::{Tag, TagType};

#[derive(Debug, Clone)]
pub struct Chunk {
    pub data_version: i32,
    pub x_pos: i32,
    pub z_pos: i32,
    pub y_pos: i32,
    pub status: String,
    pub last_update: i64,
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

#[derive(Debug, Clone)]
pub struct ChunkSection {
    y: i8,
    block_states: BlockStates,
    biomes: Biomes,
    block_light: [u8; 4096],
    sky_light: [u8; 4096],
}

impl Chunk {
    pub fn new(bytes: Vec<u8>) -> Self {
        let mut chunk = Chunk {
            data_version: 0,
            x_pos: 0,
            z_pos: 0,
            y_pos: 0,
            status: String::new(),
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

        let raw_bytes = Chunk::decompress(bytes);

        let nbt = NBT::new(&raw_bytes);

        chunk.process_chunk(nbt);

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
                    Ok(_) => { decompressed }
                    Err(err) => {
                        println!("Error: {:?}", err);
                        exit(42069);
                    }
                }
            }
            2 => {
                let mut decoder = ZlibDecoder::new(&raw_bytes as &[u8]);
                match decoder.read_to_end(&mut decompressed) {
                    Ok(_) => { decompressed }
                    Err(err) => {
                        println!("Error: {:?}", err);
                        exit(42069);
                    }
                }
            }
            _ => {
                bytes.to_vec()
            }
        };
    }

    fn process_chunk(&mut self, nbt: NBT) -> &mut Self {
        let mut missing: Vec<String> = vec![];
        for tag in nbt.tags.subtags {
            match tag.name.as_str() {
                "DataVersion" => self.data_version = tag.payload_int(),
                "xPos" => self.x_pos = tag.payload_int(),
                "yPos" => self.y_pos = tag.payload_int(),
                "zPos" => self.z_pos = tag.payload_int(),
                "Status" => self.status = tag.payload_string(),
                "LastUpdate" => self.last_update = tag.payload_long(),
                "sections" => {
                    self.sections = Chunk::process_sections(tag.subtags);
                }
                "structures" => {
                    self.structures = Chunk::process_structures(tag.subtags);
                }
                "entities" => {
                    // println!("{:?}: {:?}", tag.name, tag.tagtype); exit(42069);
                }
                "Heightmaps" => {
                    // println!("{:?}: {:?}", tag.name, tag.tagtype); exit(42069);
                }
                "Lights" => {
                    // println!("{:?}: {:?}", tag.name, tag.tagtype); exit(42069);
                }
                "isLightOn" => {
                    // println!("{:?}: {:?}", tag.name, tag.tagtype); exit(42069);
                }
                "PostProcessing" => {
                    // println!("{:?}: {:?}", tag.name, tag.tagtype); exit(42069);
                }
                "CarvingMasks" => {
                    // println!("{:?}: {:?}", tag.name, tag.tagtype); exit(42069);
                }
                "block_entities" => {
                    // println!("{:?}: {:?}", tag.name, tag.tagtype); exit(42069);
                }
                "block_ticks" => {
                    // println!("{:?}: {:?}", tag.name, tag.tagtype); exit(42069);
                }
                "fluid_ticks" => {
                    // println!("{:?}: {:?}", tag.name, tag.tagtype); exit(42069);
                }
                "InhabitedTime" => self.inhabited_time = tag.payload_long(),
                "" => {
                    // this is probably just an End tag
                }
                _ => {
                    missing.push(tag.name);
                }
            }
        }

        if 0 < missing.len() {
            println!("Missing {:?} fields: {:?}", missing.len(), missing);
            exit(42069)
        }

        self
    }

    fn process_sections(tags: Vec<Tag>) -> Vec<ChunkSection> {
        let mut sections = vec![];

        let mut missing: Vec<String> = vec![];

        for compound in tags {
            let mut section = ChunkSection {
                y: 0,
                block_states: BlockStates { palette: vec![], data: [0i16; 4096] },
                biomes: Biomes { palette: vec![], data: [0u8; 64] },
                block_light: [0u8; 4096],
                sky_light: [0u8; 4096],
            };

            for tag in compound.subtags {
                match tag.name.as_str() {
                    "Y" => {
                        section.y = tag.payload_byte();
                    }
                    "block_states" => {
                        section.block_states = Chunk::process_block_states(tag.subtags);
                    }
                    "biomes" => {
                        section.biomes = Chunk::process_biomes(tag.subtags);
                    }
                    "BlockLight" => {
                        section.block_light = Chunk::process_lights(tag.payload_byte_array());
                    }
                    "SkyLight" => {
                        section.sky_light = Chunk::process_lights(tag.payload_byte_array());
                    }
                    _ => {
                        if tag.tagtype != TagType::End {
                            missing.push(tag.name)
                        }
                    }
                }
            }

            sections.push(section);
        }

        if 0 < missing.len() {
            println!("Missing {:?} section fields: {:?}", missing.len(), missing);
            exit(42069)
        }

        sections
    }
}

#[derive(Debug, Clone)]
pub struct BlockStates {
    palette: Vec<BlockState>,
    data: [i16; 4096],
}

#[derive(Debug, Clone)]
pub struct BlockState {
    name: String,
    properties: HashMap<String, String>,
}

trait BlockStateProcessor {
    fn process_block_states(tags: Vec<Tag>) -> BlockStates;
    fn process_block_state(tag: Tag) -> BlockState;
    fn process_block_data(bits_per_entry: usize, long_ints: Vec<i64>) -> [i16; 4096];
}

impl BlockStateProcessor for Chunk {
    fn process_block_states(tags: Vec<Tag>) -> BlockStates {
        let mut missing: Vec<String> = vec![];

        let mut block_states = BlockStates {
            palette: vec![],
            data: [0i16; 4096],
        };

        // load up all palette block states
        for tag in &tags {
            match tag.name.as_str() {
                "palette" => {
                    for compound_block_tag in tag.clone().subtags {
                        block_states.palette.push(
                            Chunk::process_block_state(compound_block_tag)
                        )
                    }
                }
                "data" => {} // ignore in case the palette hasn't fully loaded
                _ => {
                    if tag.tagtype != TagType::End {
                        println!("{:?}: {:?}", tag.name, tag.tagtype);
                        missing.push(tag.clone().name)
                    }
                }
            }
        }

        // load the byte array indexing all blocks in the section
        for tag in &tags {
            match tag.name.as_str() {
                "palette" => {} // ignore because now we have all block states loaded
                "data" => {
                    // calculate index length
                    let bits_per_entry = max(
                        4,
                        (block_states.clone().palette.len() as f64).log2().ceil() as usize
                    );

                    // load block state data
                    let block_state_data = Chunk::process_block_data(
                        bits_per_entry,
                        tag.payload_long_array()
                    );

                    // convert vec to slice because why not
                    if 4096 == block_state_data.len() {
                        for i in 0..4096 {
                            block_states.data[i] = block_state_data[i];
                        }
                    }
                }
                _ => {
                    // this should never be reached
                    if !missing.contains(&tag.name) && tag.tagtype != TagType::End {
                        println!("Missing {:?} block_state fields: {:?}", missing.len(), missing);
                        exit(320)
                    }
                }
            }
        }

        if 0 < missing.len() {
            println!("Missing {:?} block_state fields: {:?}", missing.len(), missing);
            exit(535)
        }

        block_states
    }

    fn process_block_state(tag: Tag) -> BlockState {
        let mut missing = vec![];

        let mut block_state = BlockState {
            name: "".to_string(),
            properties: Default::default(),
        };

        for subtag in tag.subtags {
            match subtag.name.as_str() {
                "Name" => {
                    block_state.name = subtag.payload_string();
                }
                "Properties" => {
                    for property in subtag.subtags {
                        if TagType::End == property.tagtype { continue; }

                        block_state.properties.insert(
                            property.clone().name,
                            property.payload_string(),
                        );
                    }
                }
                _ => {
                    if subtag.tagtype != TagType::End {
                        missing.push(subtag.name)
                    }
                }
            }
        }

        if 0 < missing.len() {
            println!("Missing {:?} block_state fields: {:?}", missing.len(), missing);
            exit(535)
        }

        block_state
    }

    fn process_block_data(bits_per_entry: usize, long_ints: Vec<i64>) -> [i16; 4096] {
        let mut data: [i16; 4096] = [0i16; 4096];
        let left_trim = 64 % bits_per_entry;
        let mut indecies: Vec<i16> = vec![];

        for long_int in long_ints {
            let mut entry_list: Vec<i16> = vec![];
            let raw_bits = format!("{long_int:064b}");
            let trimmed_bits = String::from(&raw_bits[left_trim..]);

            let entries: Vec<&str> = trimmed_bits.as_bytes()
                .chunks(bits_per_entry)
                .map(std::str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap();

            for entry in entries {
                // collect all block entries and trim off remainder garbage bits
                if entry.len() == bits_per_entry {
                    entry_list.push(i16::from_str_radix(entry, 2).unwrap());
                }
            }

            // reverse the order because the documentation says so
            entry_list.reverse();

            for entry in entry_list {
                // keep pushing until indecies are maxed out and drop remaining buffer bits
                if indecies.len() < 4096 {
                    indecies.push(entry)
                }
            }
        }

        // convert vec to slice because why not
        if 4096 == indecies.len() {
            for i in 0..4096 {
                data[i] = indecies[i];
            }
        } else {
            println!("invalid data index length: {:?}\n{:?}", &indecies.len(), &indecies);
            exit(556)
        }

        data
    }
}

#[derive(Debug, Clone)]
pub struct Biomes {
    palette: Vec<String>,
    data: [u8; 64],
}

trait BiomeProcessor {
    fn process_biomes(tags: Vec<Tag>) -> Biomes;
    fn process_biome_data(bits_per_entry: usize, long_ints: Vec<i64>) -> [i16; 64];
}

impl BiomeProcessor for Chunk {
    fn process_biomes(tags: Vec<Tag>) -> Biomes {
        let mut missing: Vec<String> = vec![];

        let mut biomes = Biomes {
            palette: vec![],
            data: [0u8; 64],
        };

        // load up all palette biomes
        for tag in &tags {
            match tag.name.as_str() {
                "palette" => {
                    for subtag in &tag.subtags {
                        biomes.palette.push(subtag.payload_string());
                    }
                }
                "data" => {} // ignore in case the palette hasn't fully loaded
                _ => {
                    if tag.tagtype != TagType::End {
                        println!("{:?}: {:?}", &tag.name, &tag.tagtype);
                        missing.push(tag.clone().name)
                    }
                }
            }
        }

        // load the byte array indexing all biomes in the section
        for tag in &tags {
            match tag.name.as_str() {
                "palette" => {} // ignore because now we have all biomes loaded
                "data" => {
                    // calculate index length
                    let bits_per_entry = (biomes.palette.len() as f64).log2().ceil() as usize;

                    // load biome data
                    let biome_data = Chunk::process_biome_data(bits_per_entry, tag.payload_long_array());

                    // convert vec to slice because why not
                    if 64 == biome_data.len() {
                        for i in 0..64 {
                            biomes.data[i] = biome_data[i] as u8;
                        }
                    }
                }
                _ => {
                    // this should never be reached
                    if !missing.contains(&tag.name) && tag.tagtype != TagType::End {
                        println!("Missing {:?} biome fields: {:?}", missing.len(), missing);
                        exit(422)
                    }
                }
            }
        }

        if 0 < missing.len() {
            println!("Missing {:?} biome fields: {:?}", missing.len(), missing);
            exit(42069)
        }

        biomes
    }

    fn process_biome_data(bits_per_entry: usize, long_ints: Vec<i64>) -> [i16; 64] {
        let mut data: [i16; 64] = [0i16; 64];
        let left_trim = 64 % bits_per_entry;
        let mut indecies: Vec<i16> = vec![];

        for long_int in long_ints {
            let mut entry_list: Vec<i16> = vec![];
            let raw_bits = format!("{long_int:064b}");
            let trimmed_bits = String::from(&raw_bits[left_trim..]);

            let entries: Vec<&str> = trimmed_bits.as_bytes()
                .chunks(bits_per_entry)
                .map(std::str::from_utf8)
                .collect::<Result<Vec<&str>, _>>()
                .unwrap();

            for entry in entries {
                // collect all block entries and trim off remainder garbage bits
                if entry.len() == bits_per_entry {
                    entry_list.push(i16::from_str_radix(entry, 2).unwrap());
                }
            }

            // reverse the order because the documentation says so
            entry_list.reverse();

            for entry in entry_list {
                // keep pushing until indecies are maxed out and drop remaining buffer bits
                if indecies.len() < 4096 {
                    indecies.push(entry)
                }
            }
        }

        // convert vec to slice because why not
        if 64 == indecies.len() {
            for i in 0..64 {
                data[i] = indecies[i];
            }
        } else {
            println!("invalid data index length: {:?}\n{:?}", &indecies.len(), &indecies);
            exit(556)
        }

        data
    }
}

trait LightProcessor {
    fn process_lights(byte_array: Vec<u8>) -> [u8; 4096];
}

impl LightProcessor for Chunk {
    fn process_lights(byte_array: Vec<u8>) -> [u8; 4096] {
        // initialize the slice with zeros
        let mut block_lights: [u8; 4096] = [0; 4096];

        // iterate over each byte
        for (index, &byte) in byte_array.iter().enumerate() {
            // extract the first 4 bits
            block_lights[index * 2] = (byte >> 4) & 0b00001111;

            // extract the last 4 bits
            block_lights[index * 2 + 1] = byte & 0b00001111;
        }

        block_lights
    }
}

#[derive(Debug, Clone)]
pub struct Structure {
    structure_name: String,
    x: i32,
    z: i32,
}

trait StructureProcessor {
    fn process_structures(structure_tags: Vec<Tag>) -> Vec<Structure>;
    fn process_references(reference_tags: Vec<Tag>);
    fn process_starts(starts_tags: Vec<Tag>);
    fn process_children(children_tags: Vec<Tag>);
}

impl StructureProcessor for Chunk {
    fn process_structures(structure_tags: Vec<Tag>) -> Vec<Structure> {
        let mut structures = vec![];

        let mut missing: Vec<String> = vec![];

        for structure in structure_tags {
            // println!("{:?}: {:?}", structure.name, structure.tagtype);
            match structure.name.as_str() {
                "References" => {
                    // println!("{:?}", structure.subtags); exit(577);
                    Chunk::process_references(structure.subtags);
                }
                "starts" => {
                    // println!("{:?}", structure.subtags); //exit(580);
                    Chunk::process_starts(structure.subtags);
                }
                _ => {
                    if structure.tagtype != TagType::End {
                        missing.push(structure.name)
                    }
                }
            }
        }

        if 0 < missing.len() {
            println!("Missing {:?} section fields: {:?}", missing.len(), missing);
            exit(589)
        }

        structures
    }

    fn process_references(reference_tags: Vec<Tag>) {
        let mut missing: Vec<String> = vec![];

        let mut references: Vec<String> = vec![];

        for reference in reference_tags {
            if TagType::End == reference.tagtype { continue }

            let name = &reference.name;

            let bit_mask = 0b0000000000000000000000000000000011111111111111111111111111111111;

            let packed_coordinates = &reference.payload_long_array();

            for coordinates in packed_coordinates {
                // extract the chunk x coordinate
                let z = coordinates >> 32 & bit_mask;

                // extract the chunk z coordinate
                let x = coordinates & bit_mask;
            }

        }

        if 0 < missing.len() {
            println!("Missing {:?} reference fields: {:?}", missing.len(), missing);
            exit(589)
        }
    }

    fn process_starts(starts_tags: Vec<Tag>) {
        let mut missing: Vec<String> = vec![];

        let mut starts: Vec<String> = vec![];

        let mut id = String::new();
        let mut chunk_x = 0;
        let mut chunk_z = 0;

        for start in starts_tags {
            for subtag in &start.subtags {
                match subtag.name.as_str() {
                    "Children" => {
                        // println!("Children {:?}: {:?}", subtag.name, subtag.subtags)
                        Chunk::process_children(subtag.clone().subtags);
                    }
                    "ChunkX" => {
                        chunk_x = subtag.payload_int()
                    }
                    "ChunkZ" => {
                        chunk_z = subtag.payload_int()
                    }
                    "id" => {
                        id = subtag.payload_string()
                    }
                    "references" => {
                        println!("references {:?}: {:?}", subtag.name, subtag.payload_int())
                    }
                    _ => {
                        if subtag.tagtype != TagType::End {
                            println!("{:?}: {:?}", subtag.name, subtag.subtags);
                            missing.push(subtag.clone().name)
                        }
                    }
                }
            }
        }

        if 0 < missing.len() {
            println!("Missing {:?} starts.subtag fields: {:?}", missing.len(), missing);
            exit(589)
        }
    }

    fn process_children(children_tags: Vec<Tag>) {
        let mut missing: Vec<String> = vec![];

        let mut children: Vec<String> = vec![];

        for child in children_tags {
            for subtag in child.subtags {
                match subtag.name.as_str() {
                    "BB" => {}
                    "BiomeType" => {}
                    "D" => {}
                    "Entrances" => {}
                    "GD" => {}
                    "hps" => {}
                    "hr" => {}
                    "id" => {}
                    "Integrity" => {}
                    "isBeached" => {}
                    "IsLarge" => {}
                    "MST" => {}
                    "Num" => {}
                    "O" => {}
                    "Rot" => {}
                    "sc" => {}
                    "Template" => {}
                    "tf" => {}
                    "TPX" => {}
                    "TPY" => {}
                    "TPZ" => {}
                    _ => {
                        if subtag.tagtype != TagType::End && !missing.contains(&subtag.name){
                            println!("{:?}: {:?}", subtag.name, subtag.tagtype);
                            missing.push(subtag.clone().name)
                        }
                    }
                }
            }
        }

        if 0 < missing.len() {
            println!("Missing {:?} starts.children fields: {:?}", missing.len(), missing);
            exit(589)
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockEntity {
    entity_type: String,
    properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TileTick {
    i: Box<str>,
    p: Vec<Box<str>>,
    t: i32,
    x: i32,
    y: i32,
    z: i32,
}
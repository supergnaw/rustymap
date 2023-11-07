//! MINECRAFT REGION FILES (.mcr, .mca)
//!
//! Region files begin with an 8KiB header, split into two 4KiB tables. The first containing the
//! offsets of chunks in the region file itself, the second providing timestamps for the last
//! updates of those chunks. The remaining payload data consists of the chunk payloads

//! HEADER
//!
//! | bytes -->       |            0x00 - 0x0FFF               |             0x1000 - 0x1FFF             |        0x2000...        |
//! |-----------------|----------------------------------------|-----------------------------------------|-------------------------|
//! | description --> | locations (1024 entries; 4 bytes each) | timestamps (1024 entries; 4 bytes each) | chunks and unused space |

//! LOCATION TABLE
//!
//! Location information for a chunk consists of four bytes split into two fields: the first three
//! bytes are a (big-endian) offset in 4KiB sectors from the start of the file, and a remaining byte
//! that gives the length of the chunk (also in 4KiB sectors, rounded up). Chunks are always less
//! than 1MiB in size. If a chunk isn't present in the region file (e.g. because it hasn't been
//! generated or migrated yet), both fields are zero. A chunk with an offset of 2 begins right after
//! the timestamps table.
//!
//! | BYTES -->       | 0 | 1 | 2 |       3      |
//! |-----------------|---|---|---|--------------|
//! | DESCRIPTION --> |  offset   | sector count |

//! TIMESTAMP TABLE
//!
//! The entries in the timestamp table are individual four-byte big-endian integers, representing
//! the last modification time of a chunk in epoch seconds.
//!
//! | BYTES -->       | 0 | 1 | 2 | 3 |
//! |-----------------|---|---|---|---|
//! | DESCRIPTION --> |   timestamp   |

use std::{fs::File};
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use crate::chunk::*;

#[derive(Debug)]
pub struct Region {
    pub(crate) region_path: String,
    region_headers: HashMap<i32, RegionHeader>,
    pub chunks: Vec<Chunk>,
    pub region_x: i32,
    pub x: i32,
    pub region_z: i32,
    pub z: i32,
}

#[derive(Debug)]
pub struct RegionHeader {
    offset: u64,
    updated: u32,
    sectors: usize,
    size: usize,
}

pub trait RegionLoader {
    fn new(region_path: &str) -> Self;
    fn load_chunks(&mut self);
}

impl RegionLoader for Region {
    fn new(region_path: &str) -> Self {
        let filename_parts: Vec<&str> = region_path.split(".").collect();
        let region_x = filename_parts[1].parse::<i32>().unwrap();
        let region_z = filename_parts[2].parse::<i32>().unwrap();
        let mut region = Region {
            region_path: String::from(region_path),
            region_headers: HashMap::new(),
            chunks: Vec::new(),
            region_x: region_x,
            x: region_x * 512,
            region_z: region_z,
            z: region_z * 512,
        };
        region.load_chunks();
        region
    }


    fn load_chunks(&mut self) {
        let mut region_file = File::open(&self.region_path).unwrap();
        let mut location_buffer = vec![0u8; 4096];
        let _ = region_file.read_exact(&mut location_buffer);
        let mut updated_buffer = vec![0u8; 4096];
        let _ = region_file.read_exact(&mut updated_buffer);

        for cur in (0..4096).step_by(4) {
            // get updated timestamp
            let slice: [u8; 4] = updated_buffer[cur..cur + 4].try_into().unwrap();
            let updated = u32::from_be_bytes(slice);

            // get byte offset
            let slice: [u8; 4] = [0, location_buffer[cur], location_buffer[cur+1], location_buffer[cur+2]];
            let offset = u64::from(u32::from_be_bytes(slice) * 4096);

            // get chunk sector count
            let mut slice = [0u8; 4];
            slice[3] = location_buffer[cur + 3].try_into().unwrap();
            let sectors = u32::from_be_bytes(slice) as usize;
            let size = &sectors * 4096;

            // non-generated chunk
            // if 0 == updated && 0 == size { continue; }

            // save chunk to table header
            let chunk_header = RegionHeader { offset, updated, sectors, size, };
            self.region_headers.insert((cur as i32) / 4, chunk_header);
        }

        // let region = &self.region_headers["3676"];
        for header in &self.region_headers {
            let region = header.1;
            if 0 == region.size { continue; }
            let mut chunk_buffer = vec![0u8; region.size];
            match region_file.seek(SeekFrom::Start(region.offset)) {
                Ok(_) => {}
                Err(err) => { format!("Failed to find file offset: {:?}", err); () }
            }
            match region_file.read_exact(&mut chunk_buffer) {
                Ok(()) => {},
                Err(err) => { format!("Failed to read chunk bytes: {:?}", err ); () }
            }
        }

        let mut r: i32 = 0;
        for (r, region_header) in &self.region_headers {
            let offset = u64::from(region_header.offset) ;
            let _ = region_file.seek(SeekFrom::Start(offset));
            let mut chunk_buffer = vec![0u8; region_header.size];
            let _ = region_file.read_exact(&mut chunk_buffer);
            let x = self.x + (r % 32 * 16);
            let z = self.z + (r / 32 * 16);
            let chunk = Chunk::new(chunk_buffer, x, z);
            self.chunks.push(chunk);
        }

        println!(" - loaded {:?} chunks", &self.region_headers.len());
    }
}
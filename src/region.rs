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
    region_path: String,
    region_headers: HashMap<String, RegionHeader>,
    pub chunks: Vec<Chunk>,
}

#[derive(Debug)]
pub struct RegionHeader {
    offset: u64,
    updated: u32,
    sectors: u32,
    size: usize,
}

pub trait RegionLoader {
    fn new(region_path: &str) -> Self;
    fn load_chunks(&mut self);
}

impl RegionLoader for Region {
    fn new(region_path: &str) -> Self {
        let mut region = Region {
            region_path: String::from(region_path),
            region_headers: HashMap::new(),
            chunks: Vec::new(),
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

        for i in 0..1024 {
            let cur = i * 4;

            // get timestamp
            let slice: [u8; 4] = updated_buffer[cur..cur + 4].try_into().unwrap();
            let timestamp = u32::from_be_bytes(slice);
            if 0 == timestamp {
                continue;
            }

            // get byte offset
            let sub_slice: [u8; 3] = location_buffer[cur..cur + 3].try_into().unwrap();
            let mut slice = [0u8; 4];
            slice[1..].copy_from_slice(&sub_slice);
            let offset: u64 = (u32::from_be_bytes(slice) * 4096).into();
            if 0 == offset {
                continue;
            }

            // get chunk size data
            let mut slice = [0u8; 4];
            slice[3] = location_buffer[cur + 3].try_into().unwrap();
            let sectors = u32::from_be_bytes(slice);
            let size = (sectors * 4096) as usize;

            // save chunk to table header
            let chunk_header = RegionHeader {
                offset: offset,
                updated: timestamp,
                sectors: sectors,
                size: size,
            };
            self.region_headers.insert(i.to_string(), chunk_header);
        }

        for region_header in &self.region_headers {
            let offset = u64::from(region_header.1.offset);
            let _ = region_file.seek(SeekFrom::Start(offset));
            let mut chunk_buffer = vec![0u8; region_header.1.size];
            let _ = region_file.read_exact(&mut chunk_buffer);
            let chunk = Chunk::new(chunk_buffer);
            self.chunks.push(chunk);
        }

        println!(" - loaded {:?} chunks", &self.region_headers.len());
    }
}

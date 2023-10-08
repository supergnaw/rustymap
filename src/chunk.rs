//! CHUNK
//!
//! Chunk payload data begins with a (big-endian) four-byte signed length field that indicates
//! the exact length of the remaining chunk data in bytes. The following byte indicates the
//! compression scheme used for chunk data, and the remaining (length-1) bytes are the compressed
//! chunk data.
//!
//! | BYTES -->       | 0  | 1  | 2  | 3  |         4        |                 5                |
//! |-----------------|-------------------|------------------|----------------------------------|
//! | DESCRIPTION --> | length (in bytes) | compression type | compressed data (length-1 bytes) |bit.

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

use std::io::Read;
use flate2::read::{GzDecoder, ZlibDecoder};
use std::usize;
use crate::nbt::*;

#[derive(Debug)]
pub struct Chunk {
    length: u32,
    compression_type: usize,
    pub nbt: NBT,
}

pub trait ChunkLoader {
    fn new(bytes: Vec<u8>) -> Self;


    fn decompress(raw_bytes: &Vec<u8>, compression_type: &usize) -> Vec<u8>;
}

impl ChunkLoader for Chunk {
    fn new(bytes: Vec<u8>) -> Self {
        let len: u32 = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let comp_type: usize = usize::from(bytes[4]);
        let raw_bytes: Vec<u8> = bytes[5..4 + len as usize].to_vec();
        let payload = Chunk::decompress(&raw_bytes, &comp_type);
        let nbt: NBT = NBT::new(&payload);
        let chunk = Chunk {
            length: len,
            compression_type: comp_type,
            nbt: nbt,
        };
        chunk
    }


    fn decompress(raw_bytes: &Vec<u8>, compression_type: &usize) -> Vec<u8> {
        let mut decompressed = Vec::new();
        match compression_type {
            1 => {
                let mut decoder = GzDecoder::new(&raw_bytes as &[u8]);
                let _ = decoder.read_to_end(&mut decompressed);
                return decompressed;
            }
            2 => {
                let mut decoder = ZlibDecoder::new(&raw_bytes as &[u8]);
                let _ = decoder.read_to_end(&mut decompressed);
                return decompressed;
            }
            _ => {
                decompressed = raw_bytes.to_vec();
                return decompressed;
            }
        }
    }
}
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use sha1::{Sha1, Digest};
use crate::chunk::Chunk;
use crate::region::Region;

// probably use ron
// https://docs.rs/serde/1.0.192/serde/
// https://github.com/ron-rs/ron
// https://serde.rs/#data-formats
// https://blog.ediri.io/serialize-and-deserialize-data-in-rust-using-serde-and-serdejson

#[derive(Debug)]
pub struct Cache {
    pub cache_dir: PathBuf,
}

impl Cache {
    pub fn new(cache_dir: PathBuf) -> Self {
        let mut cache = Cache {
            cache_dir: PathBuf::from(cache_dir)
        };
        cache.make_cache_dir();

        cache
    }

    fn make_cache_dir(&self) {
        if self.cache_dir.exists() { return () }

        match fs::create_dir_all(&self.cache_dir.as_path()) {
            Ok(_) => {} // success
            Err(err) => {
                eprintln!(
                    "Failed to create cache directory ({:?}): {err}",
                    &self.cache_dir.as_path()
                );
                exit(26)
            }
        }
    }

    fn load(self, path: String) {
        let mut path_buf = self.cache_dir;
        // return Option<Some, None>
    }

    fn save(self, file_path: String, data: Vec<u8>) {
        println!("Save to {:?}: {:?}", &file_path, &data);
    }

    fn hash_string(input: String) -> String {
        let mut sha1 = Sha1::new();
        sha1.update(input);
        let result = sha1.finalize();
        format!("{result:x}")
    }
}

pub trait RegionCache {
    fn load_region(&mut self, x: i32, z: i32) -> Region;
    fn save_region(&mut self, region: Region);
}

impl RegionCache for Cache {
    fn load_region(&mut self, x: i32, z: i32) -> Region {
        todo!()
    }

    fn save_region(&mut self, region: Region) {
        todo!()
    }
}

pub trait ChunkCache {
    fn load_chunk(&mut self, x: i32, z: i32) -> Chunk;
    fn save_chunk(&mut self, chunk: Chunk);
}

impl ChunkCache for Cache {
    fn load_chunk(&mut self, x: i32, z: i32) -> Chunk {
        todo!()
    }

    fn save_chunk(&mut self, chunk: Chunk) {
        todo!()
    }
}

pub trait JarCache {
    fn load_blockstate(&mut self, block: String);
    fn load_model(&mut self, block: String);
}

impl JarCache for Cache {
    fn load_blockstate(&mut self, block: String) {
        todo!()
    }

    fn load_model(&mut self, block: String) {
        todo!()
    }
}

pub trait TextureCache {
    fn load_block_texture(&mut self, block: String);
    fn load_entity_texture(&mut self, entity: String);
    fn load_item_texture(&mut self, item: String);
}

impl TextureCache for Cache {
    fn load_block_texture(&mut self, block: String) {
        todo!()
    }

    fn load_entity_texture(&mut self, entity: String) {
        todo!()
    }

    fn load_item_texture(&mut self, item: String) {
        todo!()
    }
}
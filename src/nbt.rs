use crate::tag::*;

#[derive(Debug)]
pub struct NBT {
    bytes: Vec<u8>,
    tags: Vec<Tag>,
    cur: usize,
}

pub trait NBTLoader {
    fn new(bytes: &Vec<u8>) -> Self;
    fn parse_tag(&mut self);
}

impl NBTLoader for NBT {
    fn new(bytes: &Vec<u8>) -> Self {
        let mut nbt = NBT {
            bytes: bytes.to_vec(),
            cur: 0,
            tags: Vec::new(),
        };
        nbt.parse_tag();
        nbt
    }


    fn parse_tag(&mut self) {
        while (self.cur) < self.bytes.len() {
            let bytes: Vec<u8> = self.bytes[(self.cur as usize)..].to_vec();
            let tag = Tag::new(bytes);
            self.cur += tag.cur;
            self.tags.push(tag);
        }
    }
}

use crate::tag::*;

#[derive(Debug)]
pub struct NBT {
    bytes: Vec<u8>,
    pub tags: Vec<Tag>,
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
        // dbg!(some_variable.payload.read());
        nbt
    }


    fn parse_tag(&mut self) {
        while (self.cur) < self.bytes.len() {
            let bytes: Vec<u8> = self.bytes[(self.cur)..].to_vec();
            let proto_tag = ProtoTag::new(bytes);
            self.cur += proto_tag.cur;
            let tag = Tag {
                name: proto_tag.name,
                payload: proto_tag.payload,
            };
            self.tags.push(tag);
        }
    }
}

use crate::tag::*;

#[derive(Debug)]
pub struct NBT {
    pub(crate) tags: Tag,
}

impl NBT {
    pub fn new(bytes: &Vec<u8>) -> Self {
        NBT {
            tags: Tag::new(bytes.clone()),
        }
    }
}
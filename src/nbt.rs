use crate::tag::*;

#[derive(Debug)]
pub struct NBT {
    pub(crate) tags: Tag,
}

impl NBT {
    pub fn new(bytes: &Vec<u8>) -> Self {
        match bytes.len() {
            0 => NBT { tags: Tag::new(vec![99]) },
            _ => NBT { tags: Tag::new(bytes.clone()) },
        }
    }
}
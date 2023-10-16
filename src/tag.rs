//! STRUCTURE TAGS
//!
//! A tag is an individual part of the data tree. The first byte in a tag is the tag type (ID),
//! followed by a two byte big-endian unsigned integer for the length of the name, then the name as
//! a string in UTF-8 format (Note TAG_End is not named and does not contain the extra 2 bytes; the
//! name is assumed to be empty). Finally, depending on the type of the tag, the bytes that follow
//! are part of that tag's payload.
//!
//! | BYTES -->       | 0  |   1  |   2  |    3..n     | n+1.. |
//! |-----------------|----|-------------|-------------|-------|
//! | DESCRIPTION --> | id | name length | name utf-8  | data  |

use std::cmp::min;
use std::process::exit;

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub struct Tag {
    pub name: String,
    pub payload: TagPayload,
}

impl Tag {
    pub fn new(bytes: Vec<u8>) -> Self {
        let proto_tag = ProtoTag::new(bytes);

        Tag::collapse(proto_tag)
    }

    fn collapse(proto_tag: ProtoTag) -> Tag {
        Tag {
            name: proto_tag.name,
            payload: proto_tag.payload,
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub struct ProtoTag {
    name: String,
    payload: TagPayload,
    tag_type: TagType,
    cur: usize,
    raw: Vec<u8>,
}

impl ProtoTag {
    fn new(mut bytes: Vec<u8>) -> Self {
        // create a tag
        let mut proto_tag = ProtoTag {
            name: "".to_string(),
            payload: TagPayload::Invalid,
            tag_type: TagType::Invalid,
            cur: 0,
            raw: bytes,
        };

        // process the tag bytes
        proto_tag.process();

        // return the tag
        proto_tag
    }

    fn read_tag_header(bytes: &Vec<u8>) -> (TagType, String, usize, Vec<u8>) {
        let id_byte = bytes[0];
        let tag_type = ProtoTag::id_type(&id_byte);

        match tag_type {
            TagType::End => return (tag_type, String::new(), 1, [bytes[0]].to_vec()),
            TagType::Invalid => exit(42069),
            _ => {}
        }

        let name_len = bytes_to_usize(&bytes[1..=2], false);
        let char_bytes = bytes[3..3 + name_len].to_vec();
        (tag_type, bytes_to_utf8(&char_bytes), 3 + name_len, bytes[0..3 + name_len].to_vec())
    }

    fn id_type(id: &u8) -> TagType {
        // return tag type from id
        match id {
            0 => TagType::End,
            1 => TagType::Byte,
            2 => TagType::Short,
            3 => TagType::Int,
            4 => TagType::Long,
            5 => TagType::Float,
            6 => TagType::Double,
            7 => TagType::ByteArray,
            8 => TagType::String,
            9 => TagType::List,
            10 => TagType::Compound,
            11 => TagType::IntArray,
            12 => TagType::LongArray,
            _ => TagType::Invalid,
        }
    }

    fn process(&mut self) -> &mut ProtoTag {
        let bytes = &self.raw;
        let tag_header = ProtoTag::read_tag_header(&bytes);

        self.tag_type = tag_header.0;
        self.name = tag_header.1;
        self.cur += tag_header.2;

        self.payload = match tag_header.0 {
            // End of compound tag/no payload
            TagType::End => {
                TagPayload::End
            }
            // 1 byte / 8 bits, signed
            TagType::Byte => {
                let value = bytes_to_isize(
                    &bytes[self.cur..=self.cur], false
                ) as i8;
                self.cur += 1;
                TagPayload::Byte(value)
            }
            // 2 bytes / 16 bits, signed, big endian
            TagType::Short => {
                let value = bytes_to_isize(
                    &bytes[self.cur..self.cur + 2], false
                ) as i16;
                self.cur += 2;
                TagPayload::Short(value)
            }
            // 4 bytes / 32 bits, signed, big endian
            TagType::Int => {
                let value = bytes_to_isize(
                    &bytes[self.cur..self.cur + 4], false
                ) as i32;
                self.cur += 4;
                TagPayload::Int(value)
            }
            // 8 bytes / 64 bits, signed, big endian
            TagType::Long=> {
                let value = bytes_to_isize(
                    &bytes[self.cur..self.cur + 8], false
                ) as i64;
                self.cur += 8;
                TagPayload::Long(value)
            }
            // 4 bytes / 32 bits, signed, big endian, IEEE 754-2008, binary32
            TagType::Float => {
                let value = f32::from_be_bytes(
                    (&bytes[self.cur..self.cur + 4]).try_into().unwrap()
                );
                self.cur += 4;
                TagPayload::Float(value)
            }
            // 8 bytes / 64 bits, signed, big endian, IEEE 754-2008, binary64
            TagType::Double => {
                let value = f64::from_be_bytes(
                    (&bytes[self.cur..self.cur + 8]).try_into().unwrap()
                );
                self.cur += 8;
                TagPayload::Double(value)
            }
            // A signed integer (4 bytes) size, then the bytes comprising an array of length size.
            TagType::ByteArray => {
                let size = bytes_to_isize(&bytes[self.cur..self.cur + 4], false) as usize;
                self.cur += 4;
                let value = bytes[self.cur..self.cur + (size)].to_vec();
                self.cur += size;
                TagPayload::ByteArray(value)
            }
            // An unsigned short (2 bytes) length, then a UTF-8 string resembled by length bytes.
            TagType::String => {
                let length = bytes_to_usize(&bytes[self.cur..self.cur + 2], false);
                self.cur += 2;
                let value = bytes_to_utf8(&bytes[self.cur..self.cur + length]);
                self.cur += length;
                TagPayload::String(value)
            }
            // 1 byte of tag ID, 4 bytes signed as count, then count tags of ID
            TagType::List => {
                let id_byte = bytes[self.cur].clone();
                self.cur += 1;
                let count = bytes_to_usize(&bytes[self.cur..self.cur + 4], false);
                self.cur += 4;
                let mut value: Vec<Tag> = vec![];
                for l in 0..count {
                    // pseudo header bytes
                    let mut sub_bytes = [id_byte, 0, 0].to_vec();
                    sub_bytes.extend(bytes[self.cur..].to_vec());
                    let sub_tag = ProtoTag::new(sub_bytes);
                    // print!("DEBUG | list[{:?}]: {:?}", &l, &sub_tag);
                    self.cur += sub_tag.cur - 3;
                    // value.push(sub_tag.clone());
                    value.push(Tag { name: sub_tag.clone().name, payload: sub_tag.clone().payload });
                }
                TagPayload::List(value)
            }
            // A set of tags that continue until Tag::End
            TagType::Compound => {
                let mut value: Vec<Tag> = vec![];
                loop {
                    let sub_bytes = &bytes[self.cur..].to_vec();
                    let sub_tag = ProtoTag::new(sub_bytes.clone());
                    self.cur += sub_tag.cur;
                    // value.push(sub_tag.clone());
                    let tag_type = sub_tag.tag_type;
                    value.push(Tag { name: sub_tag.name, payload: sub_tag.payload });
                    if TagType::End == tag_type {
                        break;
                    }
                }
                TagPayload::Compound(value)
            }
            // A signed integer size, then size number of Tag::Int's payloads.
            TagType::IntArray => {
                let size = bytes_to_isize(&bytes[self.cur..self.cur + 4], false);
                self.cur += 4;
                let mut value: Vec<i32> = vec![];
                for _ in 0..size {
                    value.push(
                        bytes_to_isize(&bytes[self.cur..self.cur + 4], false
                    ) as i32);
                    self.cur += 4;
                }
                TagPayload::IntArray(value)
            }
            // A signed integer size, then size number of Tag::Long's payloads.
            TagType::LongArray => {
                let size = bytes_to_isize(&bytes[self.cur..self.cur + 4], false);
                self.cur += 4;
                let mut value: Vec<i64> = vec![];
                for _ in 0..size {
                    value.push(
                        bytes_to_isize(&bytes[self.cur..self.cur + 8], false
                    ) as i64);
                    self.cur += 8;
                }
                TagPayload::LongArray(value)
            }
            // Invalid payload
            _ => {
                TagPayload::Invalid
            }
        };

        self
    }
}

fn bytes_to_utf8<X>(bytes: &[X]) -> String where Vec<u8>: for<'a> From<&'a [X]> {
    if 0 == bytes.len() { return String::new(); }

    let bytes: Vec<u8> = bytes.into();

    return match String::from_utf8(bytes.clone()) {
        Ok(utf8) => return utf8,
        Err(err) => {
            let mut sub_bytes: [u8; 64] = [0u8; 64];
            let min_length = min(sub_bytes.len(), bytes.len());
            sub_bytes[..min_length].copy_from_slice(&bytes[..min_length]);
            let error = format!("Error trying to convert bytes to UTF8: {:?}\nbytes: {:?}", err, sub_bytes);
            println!("{:?}", &error);
            error
        }
    };
}

fn bytes_to_isize<X>(bytes: &[X], little_endian: bool) -> isize where X: Copy + Into<u8> {
    match bytes.len() {
        1 => {
            let mut slice: [u8; 1] = [0u8; 1];
            match little_endian {
                true => {
                    for (index, &value) in bytes.iter().enumerate().rev() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                },
                false => {
                    for (index, &value) in bytes.iter().enumerate() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                }
            }
            i8::from_be_bytes(slice) as isize
        }
        2 => {
            let mut slice: [u8; 2] = [0u8; 2];
            match little_endian {
                true => {
                    for (index, &value) in bytes.iter().enumerate().rev() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                },
                false => {
                    for (index, &value) in bytes.iter().enumerate() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                }
            }
            i16::from_be_bytes(slice) as isize
        }
        3..=4 => {
            let mut slice: [u8; 4] = [0u8; 4];
            match little_endian {
                true => {
                    for (index, &value) in bytes.iter().enumerate().rev() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                },
                false => {
                    for (index, &value) in bytes.iter().enumerate() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                }
            }
            i32::from_be_bytes(slice) as isize
        }
        5..=8 => {
            let mut slice: [u8; 8] = [0u8; 8];
            match little_endian {
                true => {
                    for (index, &value) in bytes.iter().enumerate().rev() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                },
                false => {
                    for (index, &value) in bytes.iter().enumerate() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                }
            }
            i64::from_be_bytes(slice) as isize
        }
        _ => {
            println!("Value is outside the specified isize ranges: {:?}", &bytes.len());
            exit(42069);
        }
    }
}

fn bytes_to_usize<X>(bytes: &[X], little_endian: bool) -> usize where X: Copy + Into<u8> {
    match bytes.len() {
        1 => {
            let mut slice: [u8; 1] = [0u8; 1];
            match little_endian {
                true => {
                    for (index, &value) in bytes.iter().enumerate().rev() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                },
                false => {
                    for (index, &value) in bytes.iter().enumerate() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                }
            }
            u8::from_be_bytes(slice) as usize
        }
        2 => {
            let mut slice: [u8; 2] = [0u8; 2];
            match little_endian {
                true => {
                    for (index, &value) in bytes.iter().enumerate().rev() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                },
                false => {
                    for (index, &value) in bytes.iter().enumerate() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                }
            }
            u16::from_be_bytes(slice) as usize
        }
        3..=4 => {
            let mut slice: [u8; 4] = [0u8; 4];
            match little_endian {
                true => {
                    for (index, &value) in bytes.iter().enumerate().rev() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                },
                false => {
                    for (index, &value) in bytes.iter().enumerate() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                }
            }
            u32::from_be_bytes(slice) as usize
        }
        5..=8 => {
            let mut slice: [u8; 8] = [0u8; 8];
            match little_endian {
                true => {
                    for (index, &value) in bytes.iter().enumerate().rev() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                },
                false => {
                    for (index, &value) in bytes.iter().enumerate() {
                        slice[index + (slice.len() - bytes.len())] = value.into();
                    }
                }
            }
            u64::from_be_bytes(slice) as usize
        }
        _ => {
            println!("Value is outside the specified usize ranges: {:?}", &bytes.len());
            exit(42069);
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum TagType {
    End,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    ByteArray,
    String,
    List,
    Compound,
    IntArray,
    LongArray,
    Invalid,
}

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum TagPayload {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Tag>),
    Compound(Vec<Tag>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    Invalid,
}

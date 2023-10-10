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
use std::i32;
use std::process::exit;
use crate::DEBUG;

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
pub struct Tag {
    pub name: String,
    pub payload: TagPayload,
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
    List(Vec<TagPayload>),
    Compound(Vec<Tag>),
    IntArray(Vec<i16>),
    LongArray(Vec<i64>),
    Invalid,
}

pub trait TagLoader {
    fn new(bytes: Vec<u8>) -> Self;
}

impl TagLoader for Tag {
    fn new(bytes: Vec<u8>) -> Self {
        let prototag = ProtoTag::new(bytes);
        let tag = Tag {
            name: prototag.name,
            payload: prototag.payload,
        };
        tag
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ProtoTag {
    pub(crate) tag_type: TagType,
    pub name: String,
    pub payload: TagPayload,
    pub cur: usize,
    cur_hist: Vec<usize>,
    bytes: Vec<u8>,
}

pub trait ProtoTagLoader {
    fn new(tag_bytes: Vec<u8>) -> Self;
    fn parse(&mut self);
    fn move_cur(&mut self, num: usize, from_start: bool);
    fn read_bytes(&mut self, num: usize) -> Vec<u8>;
    fn read_type(&mut self) -> TagType;
    fn tag_from_id(id: usize) -> TagType;
    fn read_utf8(&mut self, len: usize) -> String;
    fn bytes_to_utf8(bytes: Vec<u8>) -> String;
    fn read_name(&mut self) -> String;
    fn read_payload(&mut self, tag_type: TagType) -> TagPayload;
}

impl ProtoTagLoader for ProtoTag {
    fn new(tag_bytes: Vec<u8>) -> Self {
        let mut proto_tag = ProtoTag {
            bytes: tag_bytes,
            tag_type: TagType::Invalid,
            name: String::new(),
            payload: TagPayload::Invalid,
            cur: 0,
            cur_hist: vec![0],
        };
        proto_tag.parse();
        proto_tag
    }

    fn parse(&mut self) {
        if DEBUG { println!("parse() called") };
        self.tag_type = self.read_type();
        if TagType::End == self.tag_type {
            self.payload = self.read_payload(TagType::End);
            return;
        }

        self.name = self.read_name();
        self.payload = self.read_payload(self.tag_type);

        return;
    }

    fn move_cur(&mut self, num: usize, from_start: bool) {
        if DEBUG { println!("move_cur({:?}, {:?}) called", &num, &from_start) };
        self.cur_hist.push(num);
        match from_start {
            true => {
                self.cur = num;
            }
            false => {
                self.cur += num;
            }
        }
    }

    fn read_bytes(&mut self, num: usize) -> Vec<u8> {
        if DEBUG { println!("read_bytes({:?}) called", &num) };
        // no need to read, return an empty vec
        if 0 == num { return Vec::new(); }

        // get the start of the next range of bytes to read
        let start = self.cur;

        // calculate the total size of the range
        let end = min(start + num, self.bytes.len());

        // load the bytes into it's own vec
        let vec = Vec::<u8>::from(&self.bytes[start..end]);

        // adjust the cursor accordingly
        self.move_cur(num, false);

        return vec;
    }

    fn read_type(&mut self) -> TagType {
        if DEBUG { println!("read_type() called") };
        let byte = self.read_bytes(1)[0] as usize;
        return match byte {
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
        };
    }


    fn tag_from_id(id: usize) -> TagType {
        if DEBUG { println!("tag_from_id({:?}) called", &id) };
        return match id {
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
        };
    }

    fn read_utf8(&mut self, len: usize) -> String {
        if DEBUG { println!("read_utf8({:?}) called", &len) };
        if 0 == len {
            return String::new();
        }

        let bytes = self.read_bytes(len);
        match String::from_utf8((*bytes).to_vec()) {
            Ok(utf8) => {
                return utf8;
            }
            Err(err) => {
                dbg!(&len);
                dbg!(&self.tag_type);
                dbg!(&self.name);
                dbg!(&self.payload);
                dbg!(&self.cur_hist);
                println!("bytes: {:?}", self.bytes[0..100].to_vec());
                println!("Error??? {:?}", err);
                exit(42069);
            }
        }
    }

    fn bytes_to_utf8(bytes: Vec<u8>) -> String {
        if DEBUG { println!("bytes_to_utf8({:?}) called", bytes) };
        return match bytes.len() {
            0 => String::from(""),
            _ => {
                match String::from_utf8((*bytes).to_vec()) {
                    Ok(utf8) => return utf8,
                    Err(_) => {
                        let mut sub_bytes: [u8; 64] = [0u8; 64];
                        let min_length = min(sub_bytes.len(), bytes.len());
                        sub_bytes[..min_length].copy_from_slice(&bytes[..min_length]);
                        exit(42069);
                    }
                }
            }
        };
    }

    fn read_name(&mut self) -> String {
        let len = u16::from_be_bytes(
            // todo: add error handling
            self.read_bytes(2).try_into().unwrap()
        ) as usize;
        self.read_utf8(len)
    }

    fn read_payload(&mut self, tag_type: TagType) -> TagPayload {
        return match tag_type {
            // End of compound tag/no payload
            TagType::End => {
                TagPayload::End
            }
            // 1 byte / 8 bits, signed
            TagType::Byte => {
                let bytes = self.read_bytes(1);
                // todo: add error handling
                let value = i8::from_be_bytes(bytes.try_into().unwrap());
                TagPayload::Byte(value)
            }
            // 2 bytes / 16 bits, signed, big endian
            TagType::Short => {
                let bytes = self.read_bytes(2);
                // todo: add error handling
                let value = i16::from_be_bytes(bytes.try_into().unwrap());
                TagPayload::Short(value)
            }
            // 4 bytes / 32 bits, signed, big endian
            TagType::Int => {
                let bytes = self.read_bytes(4);
                // todo: add error handling
                let value = i32::from_be_bytes(bytes.try_into().unwrap());
                TagPayload::Int(value)
            }
            // 8 bytes / 64 bits, signed, big endian
            TagType::Long => {
                let bytes = self.read_bytes(8);
                // todo: add error handling
                let value = i64::from_be_bytes(bytes.try_into().unwrap());
                TagPayload::Long(value)
            }
            // 4 bytes / 32 bits, signed, big endian, IEEE 754-2008, binary32
            TagType::Float => {
                let bytes = self.read_bytes(4);
                // todo: add error handling
                let value = f32::from_be_bytes(bytes.try_into().unwrap());
                TagPayload::Float(value)
            }
            // 8 bytes / 64 bits, signed, big endian, IEEE 754-2008, binary64
            TagType::Double => {
                let bytes = self.read_bytes(8);
                // todo: add error handling
                let value = f64::from_be_bytes(bytes.try_into().unwrap());
                TagPayload::Double(value)
            }
            // A signed integer (4 bytes) size, then the bytes comprising an array of length size.
            TagType::ByteArray => {
                let payload_size = i32::from_be_bytes(
                    // todo: add error handling
                    self.read_bytes(4).try_into().unwrap()
                ) as usize;
                let value = self.read_bytes(payload_size);
                TagPayload::ByteArray(value)
            }
            // An unsigned short (2 bytes) length, then a UTF-8 string resembled by length bytes.
            TagType::String => {
                let payload_size = u16::from_be_bytes(
                    // todo: add error handling
                    self.read_bytes(2).try_into().unwrap()
                ) as usize;
                let value: String = self.read_utf8(payload_size);
                TagPayload::String(value)
            }
            // 1 byte of tag ID, 4 bytes signed as tag_count, then tag_count tags of ID
            TagType::List => {
                // get the tag type id in the list
                let subtag_id = self.read_bytes(1)[0];

                // get the tag type type
                let subtag_type = ProtoTag::tag_from_id(subtag_id as usize);

                // get the count of subtags in list
                let subtag_count = i32::from_be_bytes(
                    // todo: add error handling
                    self.read_bytes(4).try_into().unwrap()
                ) as isize;

                // prepare an empty list for the tags
                let mut value: Vec<TagPayload> = vec![];

                // do a barrel roll!
                for _ in 0..subtag_count {
                    let subtag = self.read_payload(subtag_type);
                    value.push(subtag);
                }
                TagPayload::List(value)
            }
            // A set of tags that continue until Tag::End
            TagType::Compound => {
                let mut tags = vec![];

                loop {
                    let tag_type = self.read_type();

                    // this compound tag has ended
                    if TagType::End == tag_type {
                        // convert the self ProtoTag into a regular Tag
                        let new_tag = Tag {
                            name: "".to_string(),
                            payload: TagPayload::End,
                        };

                        // eat your veggies
                        tags.push(new_tag);

                        break;
                    }

                    let tag_name = self.read_name();

                    // Just keep swimming...
                    match tag_type {
                        // this compound tag has ended
                        TagType::End => {
                            // convert the self ProtoTag into a regular Tag
                            let new_tag = Tag {
                                name: "".to_string(),
                                payload: TagPayload::End,
                            };

                            // eat your veggies
                            tags.push(new_tag);

                            return TagPayload::Compound(tags);
                        },
                        // we have to go deeper
                        TagType::Compound => {
                            // instantiate a new ProtoTag instance to parse
                            let mut proto_tag = ProtoTag {
                                tag_type,
                                name: tag_name,
                                payload: TagPayload::Invalid,
                                cur: self.cur,
                                cur_hist: vec![0],
                                bytes: self.bytes.to_vec(),
                            };

                            // parse the tag within a tag
                            proto_tag.parse();

                            // convert from ProtoTag to Tag
                            let new_tag = Tag {
                                name: proto_tag.name,
                                payload: proto_tag.payload,
                            };

                            // adjust the cursor in accordance with the tag we just parsed
                            self.move_cur(proto_tag.cur, true);

                            // add the new tag to our collection
                            tags.push(new_tag);
                        },
                        // just your average Tag
                        _ => {
                            let payload = self.read_payload(tag_type);

                            let new_tag = Tag {
                                name: tag_name,
                                payload,
                            };

                            tags.push(new_tag);
                        }
                    }
                }

                TagPayload::Compound(tags)
            }
            // A signed integer size, then size number of TAG_Int's payloads.
            TagType::IntArray => {
                let payload_size = i32::from_be_bytes(
                    // todo: add error handling
                    self.read_bytes(4).try_into().unwrap()
                ) as usize;
                let bytes = self.read_bytes(payload_size * 4);
                let value: Vec<i16> = bytes.chunks_exact(2)
                    .map(|chunk| i16::from_ne_bytes([chunk[0], chunk[1]]))
                    .collect();
                TagPayload::IntArray(value)
            }
            // A signed integer size, then size number of TAG_Long's payloads.
            TagType::LongArray => {
                let payload_size = i32::from_be_bytes(
                    // todo: add error handling
                    self.read_bytes(4).try_into().unwrap()
                ) as usize;
                let bytes = self.read_bytes(payload_size * 8);
                let value: Vec<i64> = bytes.chunks_exact(8)
                    .map(|chunk| i64::from_ne_bytes([
                        chunk[0], chunk[1], chunk[2], chunk[3],
                        chunk[4], chunk[5], chunk[6], chunk[7],
                    ]))
                    .collect();
                TagPayload::LongArray(value)
            }
            _ => {
                exit(42069);
            }
        };
    }
}
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

#[derive(Debug, PartialOrd, PartialEq)]
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
    IntArray(Vec<i16>),
    LongArray(Vec<i64>),
    Invalid,
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct Tag {
    pub tag_type: TagType,
    pub name: String,
    pub payload: TagPayload,
    pub cur: u32,
}

pub trait TagLoader {
    fn new(bytes: Vec<u8>) -> Self;
}

impl TagLoader for Tag {
    fn new(bytes: Vec<u8>) -> Self {
        let prototag = ProtoTag::new(bytes);
        let tag = Tag {
            tag_type: prototag.tag_type,
            name: prototag.name,
            payload: prototag.payload,
            cur: prototag.cur,
        };
        println!("### {:?}", &tag);
        tag
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ProtoTag {
    pub tag_type: TagType,
    pub name: String,
    pub payload: TagPayload,
    pub cur: u32,
    bytes: Vec<u8>,
}

pub trait ProtoTagLoader {
    fn new(tag_bytes: Vec<u8>) -> Self;


    fn parse(&mut self);


    fn read_bytes(&mut self, num: u32) -> Vec<u8>;


    fn tag_from_id(id: usize) -> TagType;


    fn bytes_to_utf8(bytes: Vec<u8>) -> String;


    fn payload(&mut self);


    fn sub_payload_size(sub_tag_type: TagType) -> u32;
}

impl ProtoTagLoader for ProtoTag {
    fn new(tag_bytes: Vec<u8>) -> Self {
        let mut proto_tag = ProtoTag {
            bytes: tag_bytes,
            tag_type: TagType::Invalid,
            name: String::new(),
            payload: TagPayload::Invalid,
            cur: 0,
        };
        proto_tag.parse();
        proto_tag
    }


    fn parse(&mut self) {
        // get the tag type
        self.tag_type = ProtoTag::tag_from_id(self.read_bytes(1)[0] as usize);

        // end types are a single zero byte
        if TagType::End == self.tag_type {
            // we're done here
            self.payload();
            return;
        }

        // get the bytes used to calculate the name length
        let slice = self.read_bytes(2);

        // todo: add error handling
        // convert the name length bytes to an actual value
        let name_len = u16::from_be_bytes(slice.try_into().unwrap());

        // read name_len bytes and convert them to string
        self.name = ProtoTag::bytes_to_utf8(
            // todo: add error handling
            self.read_bytes(name_len.try_into().unwrap())
        );

        // get tag payload
        self.payload();
        return;
    }


    fn read_bytes(&mut self, num: u32) -> Vec<u8> {
        // no need to read, return an empty vec
        if 0 == num { return Vec::new(); }

        // get the start of the next range of bytes to read
        let start = self.cur as usize;

        // calculate the total size of the range
        let end = min(start + num as usize, self.bytes.len());

        // load the bytes into it's own vec
        let vec = Vec::<u8>::from(&self.bytes[start..end]);

        // adjust the cursor accordingly
        self.cur = &self.cur + num;

        return vec;
    }


    fn tag_from_id(id: usize) -> TagType {
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


    fn bytes_to_utf8(bytes: Vec<u8>) -> String {
        return match bytes.len() {
            0 => String::from(""),
            _ => {
                match String::from_utf8((*bytes).to_vec()) {
                    Ok(utf8) => return utf8,
                    Err(_) => {
                        let mut sub_bytes: [u8; 64] = [0u8; 64];
                        let min_length = min(sub_bytes.len(), bytes.len());
                        sub_bytes[..min_length].copy_from_slice(&bytes[..min_length]);
                        println!("Error trying to parse UTF8 string.\n bytes: {:?}", sub_bytes);
                        exit(42);
                    }
                }
            }
        };
    }


    fn payload(&mut self) {
        match self.tag_type {
            // End of compound tag/no payload
            TagType::End => {
                self.payload = TagPayload::End;
            }
            // 1 byte / 8 bits, signed
            TagType::Byte => {
                let payload_size = 1;
                let bytes = self.read_bytes(payload_size);
                // todo: add error handling
                let value = i8::from_be_bytes(bytes.try_into().unwrap());
                self.payload = TagPayload::Byte(value);
            }
            // 2 bytes / 16 bits, signed, big endian
            TagType::Short => {
                let payload_size = 2;
                let bytes = self.read_bytes(payload_size);
                // todo: add error handling
                let value = i16::from_be_bytes(bytes.try_into().unwrap());
                self.payload = TagPayload::Short(value);
            }
            // 4 bytes / 32 bits, signed, big endian
            TagType::Int => {
                let payload_size = 4;
                let bytes = self.read_bytes(payload_size);
                // todo: add error handling
                let value = i32::from_be_bytes(bytes.try_into().unwrap());
                self.payload = TagPayload::Int(value);
            }
            // 8 bytes / 64 bits, signed, big endian
            TagType::Long => {
                let payload_size = 8;
                let bytes = self.read_bytes(payload_size);
                // todo: add error handling
                let value = i64::from_be_bytes(bytes.try_into().unwrap());
                self.payload = TagPayload::Long(value);
            }
            // 4 bytes / 32 bits, signed, big endian, IEEE 754-2008, binary32
            TagType::Float => {
                let payload_size = 4;
                let bytes = self.read_bytes(payload_size);
                // todo: add error handling
                let value = f32::from_be_bytes(bytes.try_into().unwrap());
                self.payload = TagPayload::Float(value);
            }
            // 8 bytes / 64 bits, signed, big endian, IEEE 754-2008, binary64
            TagType::Double => {
                let payload_size = 8;
                let bytes = self.read_bytes(payload_size);
                // todo: add error handling
                let value = f64::from_be_bytes(bytes.try_into().unwrap());
                self.payload = TagPayload::Double(value);
            }
            // A signed integer (4 bytes) size, then the bytes comprising an array of length size.
            TagType::ByteArray => {
                let payload_size = i32::from_be_bytes(
                    // todo: add error handling
                    self.read_bytes(4).try_into().unwrap()
                ) as u32;
                let value = self.read_bytes(payload_size);
                self.payload = TagPayload::ByteArray(value);
            }
            // An unsigned short (2 bytes) length, then a UTF-8 string resembled by length bytes.
            TagType::String => {
                let payload_size = u16::from_be_bytes(
                    // todo: add error handling
                    self.read_bytes(2).try_into().unwrap()
                ) as u32;
                let bytes = self.read_bytes(payload_size);
                // todo: add error handling
                let value: String = ProtoTag::bytes_to_utf8(bytes.try_into().unwrap());
                self.payload = TagPayload::String(value);
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
                let mut subtags: Vec<Tag> = vec![];

                // do a barrel roll!
                for i in 0..subtag_count {
                    // this should be the same for all types, but could be causing me errors...
                    match subtag_type {
                        _ => {
                            let mut sub_bytes = vec![subtag_id, 0, 0];
                            sub_bytes.extend(self.bytes[self.cur as usize..].to_vec());
                            let sub_tag = Tag::new(sub_bytes);
                            self.cur += &sub_tag.cur;
                            subtags.push(sub_tag);
                        }
                    }
                }
                self.payload = TagPayload::List(subtags);
            }

            TagType::Compound => {
                // create a new vec for tags
                let mut value: Vec<Tag> = Vec::new();

                // get remaining bytes as  slice
                let sub_slice: Vec<u8> = Vec::<u8>::from(&self.bytes[self.cur as usize..]);

                // parse the next tag
                let next_tag: Tag = Tag::new(sub_slice);

                // adjust our current cursor position
                self.cur += next_tag.cur;

                // add the tag
                value.push(next_tag);

                // finish payload
                self.payload = TagPayload::Compound(value);
            }
            // A signed integer size, then size number of TAG_Int's payloads.
            TagType::IntArray => {
                let payload_size = i32::from_be_bytes(
                    // todo: add error handling
                    self.read_bytes(4).try_into().unwrap()
                ) as u32;
                let bytes = self.read_bytes(payload_size * 4);
                let value: Vec<i16> = bytes.chunks_exact(2)
                    .map(|chunk| i16::from_ne_bytes([chunk[0], chunk[1]]))
                    .collect();
                self.payload = TagPayload::IntArray(value);
            }
            // A signed integer size, then size number of TAG_Long's payloads.
            TagType::LongArray => {
                let payload_size = i32::from_be_bytes(
                    // todo: add error handling
                    self.read_bytes(4).try_into().unwrap()
                ) as u32;
                let bytes = self.read_bytes(payload_size * 8);
                let value: Vec<i64> = bytes.chunks_exact(8)
                    .map(|chunk| i64::from_ne_bytes([
                        chunk[0], chunk[1], chunk[2], chunk[3],
                        chunk[4], chunk[5], chunk[6], chunk[7],
                    ]))
                    .collect();
                self.payload = TagPayload::LongArray(value);
            }
            _ => {
                self.payload = TagPayload::Invalid;
            }
        }
    }


    fn sub_payload_size(sub_tag_type: TagType) -> u32 {
        return match sub_tag_type {
            TagType::End => 0,
            TagType::Byte => 1,
            TagType::Short => 2,
            TagType::Int => 4,
            TagType::Long => 8,
            TagType::Float => 4,
            TagType::Double => 8,
            _ => 0,
        };
    }
}
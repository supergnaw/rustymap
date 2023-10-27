//! TAGS
//!
//! A tag is an individual part of the data tree. The first byte in a tag is the tag type (ID),
//! followed by a two byte big-endian unsigned integer for the length of the name, then the name as
//! a string in UTF-8 format (Note TAG_End is not named and does not contain the extra 2 bytes; the
//! name is assumed to be empty). Finally, depending on the type of the tag, the bytes that follow
//! are part of that tag's payload.
//!
//! | BYTES -->       | 0  |   1  |   2  |    3..n     | n+1... |
//! |-----------------|----|-------------|-------------|--------|
//! | DESCRIPTION --> | id | name length | name utf-8  |  data  |

use std::cmp::min;
use std::process::exit;

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub struct Tag {
    pub name: String,
    pub tagtype: TagType,
    pub bytes: Vec<u8>,
    pub subtags: Vec<Tag>,
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

impl Tag {
    pub fn new(bytes: Vec<u8>) -> Self {
        let mut tag = Tag {
            name: String::new(),
            tagtype: TagType::Invalid,
            bytes: vec![],
            subtags: vec![],
        };

        match tag.process(bytes) {
            Ok(tag) => {
                tag.clone()
            }
            Err(error) => {
                println!("{error}");
                exit(42069);
            }
        }
    }

    fn process(&mut self, bytes: Vec<u8>) -> Result<&mut Tag, &'static str> {
        // get tag type
        let id_byte = bytes[0];
        self.tagtype = Tag::id_type(&id_byte);
        self.bytes.push(id_byte);

        // end tags need no further processing
        if TagType::End == self.tagtype || TagType::Invalid == self.tagtype { return Ok(self); }

        // read tag name
        let mut name_len_bytes = [0u8; 2];
        name_len_bytes.copy_from_slice(&bytes[1..3]);
        self.bytes.extend(name_len_bytes);

        let name_len = i16::from_be_bytes(name_len_bytes) as usize;

        match name_len {
            0 => {}
            _ => {
                let name_bytes = bytes.clone()[3..3 + name_len].to_vec();
                self.name = bytes_to_utf8(name_bytes.clone());
                self.bytes.extend(name_bytes);
            }
        }

        // self.cursor += self.header.len();
        let mut cursor = self.bytes.len();

        // read the payload
        match self.tagtype {
            // Edn of compound tag/no payload
            TagType::End => {

            }
            // 1 byte / 8 bits, signed
            TagType::Byte => {
                self.bytes.extend(bytes[cursor..=cursor].to_vec());
            }
            // 2 bytes / 16 bits, signed, big endian
            TagType::Short => {
                self.bytes.extend(bytes[cursor..cursor + 2].to_vec());
            }
            // 4 bytes / 32 bits, signed, big endian
            TagType::Int => {
                self.bytes.extend(bytes[cursor..cursor + 4].to_vec());
            }
            // 8 bytes / 64 bits, signed, big endian
            TagType::Long => {
                self.bytes.extend(bytes[cursor..cursor + 8].to_vec());
            }
            // 4 bytes / 32 bits, signed, big endian, IEEE 754-2008, binary32
            TagType::Float => {
                self.bytes.extend(bytes[cursor..cursor + 4].to_vec());
            }
            // 8 bytes / 64 bits, signed, big endian, IEEE 754-2008, binary64
            TagType::Double => {
                self.bytes.extend(bytes[cursor..cursor + 8].to_vec());
            }
            // A signed integer (4 bytes) size, then the bytes comprising an array of length size.
            TagType::ByteArray => {
                // read tag subheader
                let mut array_size = [0u8; 4];
                array_size.copy_from_slice(&bytes[cursor..cursor + 4]);
                self.bytes.extend(array_size.to_vec());
                cursor = self.bytes.len();

                // calculate byte array size
                let array_size = i32::from_be_bytes(array_size) as usize;

                // collect byte array
                self.bytes.extend(bytes[cursor..cursor + array_size].to_vec());
                cursor = self.bytes.len();
            }
            // An unsigned short (2 bytes) length, then a UTF-8 string resembled by length bytes.
            TagType::String => {
                // read tag subheader
                let mut str_len_bytes = [0u8; 2];
                str_len_bytes.copy_from_slice(&bytes[cursor..cursor + 2]);
                self.bytes.extend(str_len_bytes.to_vec());
                cursor = self.bytes.len();

                // calculate string length
                let str_size = u16::from_be_bytes(str_len_bytes) as usize;

                // collect string bytes
                self.bytes.extend(bytes[cursor..cursor + str_size].to_vec());
                cursor = self.bytes.len();
            }
            // 1 byte of tag ID, 4 bytes signed as count, then count tags of ID
            TagType::List => {
                // get the tag id byte
                let id_byte = bytes[cursor].clone();
                self.bytes.push(id_byte);
                cursor = self.bytes.len();

                // get size bytes from tag subheader
                let mut tag_count = [0u8; 4];
                tag_count.copy_from_slice(&bytes[cursor..cursor + 4]);
                self.bytes.extend(tag_count.to_vec());
                cursor = self.bytes.len();

                // calculate size bytes from subheader
                let count = i32::from_be_bytes(tag_count);

                // collect sub-tags
                for _ in 0..count {
                    // create subtag pseudo header bytes
                    let mut subtag_bytes = [id_byte, 0, 0].to_vec();

                    // extend subtag bytes from current bytes and cursor position
                    subtag_bytes.extend(bytes[cursor..].to_vec());

                    // process subtag
                    let subtag = Tag::new(subtag_bytes.clone());

                    if TagType::Invalid == subtag.tagtype {
                        println!("=== subtags ===");
                        for s in 0..self.subtags.len() {
                            println!("{:?}: {:?}", s, self.subtags[s]);
                        }
                        println!("=== bytes ===\n{:?}", bytes);
                        exit(42069);
                    }

                    // add subtag to our collection
                    self.bytes.extend(subtag.bytes[3..].to_vec());
                    cursor = self.bytes.len();
                    self.subtags.push(subtag);
                }
            }
            // A set of tags that continue until Tag::End
            TagType::Compound => {
                loop {
                    // get bytes comprising subtag
                    let subtag_bytes = bytes[cursor..].to_vec();

                    // process subtag
                    let subtag = Tag::new(subtag_bytes);

                    if TagType::Invalid == subtag.tagtype {
                        println!("Invalid subtag detected in compound tag {:?}", self.name);
                        exit(42069);
                    }

                    // use it's bytes as our own
                    self.bytes.extend(&subtag.bytes);

                    // adjust cursor position
                    cursor = self.bytes.len();

                    // collect new subtag
                    self.subtags.push(subtag.clone());

                    // break loop once we find the end
                    if TagType::End == subtag.tagtype { break; }
                }
            }
            // A signed integer size, then size number of Tag::Int payloads.
            TagType::IntArray => {
                // read tag subheader
                let mut int_count_bytes = [0u8; 4];
                int_count_bytes.copy_from_slice(&bytes[cursor..cursor + 4]);
                self.bytes.extend(int_count_bytes);
                cursor = self.bytes.len();

                // calculate int count
                let int_count = i32::from_be_bytes(int_count_bytes);

                // collect elements
                for _ in 0..int_count {
                    self.bytes.extend(bytes[cursor..cursor + 4].to_vec());
                    cursor = self.bytes.len();
                }
            }
            // A signed integer size, then size number of Tag::Long payloads.
            TagType::LongArray => {
                // read tag subheader
                let mut long_count_bytes = [0u8; 4];
                long_count_bytes.copy_from_slice(&bytes[cursor..cursor + 4]);
                self.bytes.extend(long_count_bytes);
                cursor = self.bytes.len();

                // calculate element count
                let long_count = i32::from_be_bytes(long_count_bytes);

                // collect elements
                for _ in 0..long_count {
                    self.bytes.extend(bytes[cursor..cursor + 8].to_vec());
                    cursor = self.bytes.len();
                }
            }
            TagType::Invalid => {
                println!("Could not determine tag type");
            }
        };

        return match self.tagtype {
            TagType::Invalid => Err("Invalid tag type"),
            _ => Ok(self)
        };
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

    fn id_string(tag_type: TagType) -> String {
        // return tag string from id
        match tag_type {
            TagType::End => String::from("End"),
            TagType::Byte => String::from("Byte"),
            TagType::Short => String::from("Short"),
            TagType::Int => String::from("Int"),
            TagType::Long => String::from("Long"),
            TagType::Float => String::from("Float"),
            TagType::Double => String::from("Double"),
            TagType::ByteArray => String::from("ByteArray"),
            TagType::String => String::from("String"),
            TagType::List => String::from("List"),
            TagType::Compound => String::from("Compound"),
            TagType::IntArray => String::from("IntArray"),
            TagType::LongArray => String::from("LongArray"),
            TagType::Invalid => String::from("Invalid"),
        }
    }

    pub fn payload_byte(&self) -> i8 {
        let bytes: Vec<u8> = self.bytes.iter().rev().cloned().collect();
        i8::from_be_bytes([bytes[0]])
    }
    pub fn payload_short(&self) -> i16 {
        let bytes: Vec<u8> = self.bytes.iter().rev().cloned().collect();
        i16::from_be_bytes([bytes[1], bytes[0]])
    }
    pub fn payload_int(&self) -> i32 {
        let bytes: Vec<u8> = self.bytes.iter().rev().cloned().collect();
        i32::from_be_bytes([bytes[3], bytes[2], bytes[1], bytes[0]])
    }
    pub fn payload_long(&self) -> i64 {
        let bytes: Vec<u8> = self.bytes.iter().rev().cloned().collect();
        i64::from_be_bytes([
            bytes[7], bytes[6], bytes[5], bytes[4], bytes[3], bytes[2], bytes[1], bytes[0]
        ])
    }
    pub fn payload_float(&self) -> f32 {
        let bytes: Vec<u8> = self.bytes.iter().rev().cloned().collect();
        f32::from_be_bytes([bytes[3], bytes[2], bytes[1], bytes[0]])
    }
    pub fn payload_double(&self) -> f64 {
        let bytes: Vec<u8> = self.bytes.iter().rev().cloned().collect();
        f64::from_be_bytes([
            bytes[7], bytes[6], bytes[5], bytes[4], bytes[3], bytes[2], bytes[1], bytes[0]
        ])
    }
    pub fn payload_byte_array(&self) -> Vec<u8> {
        let cursor = 7 + i16::from_be_bytes([self.bytes[1], self.bytes[2]]) as usize;
        self.bytes[cursor..].to_vec()
    }
    pub fn payload_string(&self) -> String {
        if TagType::End == self.tagtype {
            return String::new()
        }
        let cursor = 5 + i16::from_be_bytes([self.bytes[1], self.bytes[2]]) as usize;
        bytes_to_utf8(self.bytes[cursor..].to_vec())
    }
    pub fn payload_int_array(&self) -> Vec<i32> {
        let cursor = 7 + i16::from_be_bytes([self.bytes[1], self.bytes[2]]) as usize;
        let mut output = vec![];
        for i in (cursor..self.bytes.len()).step_by(4) {
            output.push(i32::from_be_bytes([
                self.bytes[i], self.bytes[i + 1], self.bytes[i + 2], self.bytes[i + 3]
            ]))
        }
        output
    }
    pub fn payload_long_array(&self) -> Vec<i64> {
        let cursor = 7 + i16::from_be_bytes([self.bytes[1], self.bytes[2]]) as usize;
        let mut output = vec![];
        for i in (cursor..self.bytes.len()).step_by(8) {
            output.push(i64::from_be_bytes([
                self.bytes[i + 0], self.bytes[i + 1], self.bytes[i + 2], self.bytes[i + 3],
                self.bytes[i + 4], self.bytes[i + 5], self.bytes[i + 6], self.bytes[i + 7]
            ]))
        }
        output
    }

    // not sure how to implement this, or if it's even possible
    // pub fn payload(&self) -> Option<T> {
    //     match self.tagtype {
    //         TagType::Byte => Some(self.payload_byte()),
    //         TagType::Short => Some(self.payload_short()),
    //         TagType::Int => Some(self.payload_int()),
    //         TagType::Long => Some(self.payload_long()),
    //         TagType::Float => Some(self.payload_float()),
    //         TagType::Double => Some(self.payload_double()),
    //         TagType::ByteArray => Some(self.payload_byte_array()),
    //         TagType::String => Some(self.payload_string()),
    //         TagType::IntArray => Some(self.payload_int_array()),
    //         TagType::LongArray => Some(self.payload_long_array()),
    //         _ => None
    //     }
    // }
}

fn bytes_to_utf8(bytes: Vec<u8>) -> String {
    // if no bytes, then there's no string
    if 0 == bytes.len() { return String::new(); }

    // return converted UTF8 string
    return match String::from_utf8(bytes.clone()) {
        Ok(utf8) => return utf8,
        Err(_) => {
            let mut sub_bytes: [u8; 64] = [0u8; 64];
            let min_length = min(sub_bytes.len(), bytes.len());
            sub_bytes[..min_length].copy_from_slice(&bytes[..min_length]);
            let error = format!("Error trying to convert bytes to UTF8");
            println!("{:?}", &error);
            error
        }
    };
}

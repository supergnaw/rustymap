use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

enum WorldError {

}

enum RegionError {

}

enum ChunkError {

}

enum NBTError {

}


#[derive(Debug)]
enum TagError {
    InvalidPayload,
    OtherError(String),
}

impl Display for TagError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TagError::InvalidPayload => write!(f, "Invalid payload"),
            TagError::OtherError(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl Error for TagError{}
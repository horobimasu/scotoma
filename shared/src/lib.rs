pub const WRITE_FILE_CHUNK_SIZE: usize = 4 * 1024 * 1024;

pub const KEY_LENGTH: usize = 32;

pub const ENCRYPTED_FILE_EXTENSION: &str = "SCOTOMA";

pub mod crypt;
pub mod hash;
pub mod drive;
pub mod file;
pub mod key;
pub mod random;
pub mod encode;
pub mod header;
pub mod path;

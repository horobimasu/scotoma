use rand::Rng;
use rand::distributions::{Standard, Alphanumeric};
use rand::seq::SliceRandom;

pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    rand::thread_rng()
        .sample_iter(&Standard)
        .take(len)
        .collect()
}

pub fn generate_random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn shuffle_string(str: &str) -> String {
    let mut characters = str
        .chars()
        .collect::<Vec<char>>();

    characters.shuffle(&mut rand::thread_rng());

    characters
        .iter()
        .collect()
}

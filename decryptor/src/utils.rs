use std::{fs, io};
use std::io::Write;
use std::path::PathBuf;

pub fn get_input(message: &str) -> String {
    print!("{}", message);

    io::stdout()
        .flush()
        .unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .unwrap();

    input
        .trim()
        .to_string()
}

pub fn get_encrypted_key(file_path: &PathBuf, char_set: &[u8]) -> Option<String> {
    let contents = fs::read_to_string(file_path)
        .unwrap();

    let lines = contents
        .lines()
        .filter(|line| line.len() == 100)
        .filter(|line| line.bytes().all(|byte| char_set.contains(&byte) || byte == b'0'))
        .collect::<Vec<&str>>();

    if lines.is_empty() {
        return None;
    }

    let result = lines
        .join("")
        .trim_end_matches('0')
        .to_string();

    if result.is_empty() {
        return None;
    }

    Some(result)
}

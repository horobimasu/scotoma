use std::{env, format, fs, panic};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::utils::file::find_dir_from_items;

pub static PROJECT_ROOT_DIR_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init_state() {
    let exec_dir_path = env::current_dir()
        .unwrap()
        .to_path_buf();

    let root_dir_path = find_dir_from_items(&exec_dir_path, &["decryptor", "encryptor", "panel", "shared"])
        .unwrap_or_else(|| panic!("failed to find scotoma root directory"));

    update_rust_flags(&root_dir_path);

    PROJECT_ROOT_DIR_PATH
        .set(root_dir_path)
        .ok();
}

fn update_rust_flags(root: &PathBuf) {
    let userprofile = env::var("USERPROFILE")
        .unwrap()
        .replace("\\", "/");

    let config_file_path = root.join(".cargo\\config.toml");
    let contents = fs::read_to_string(&config_file_path)
        .unwrap();

    let updated = contents
        .lines()
        .map(|line| {
            if line.contains("--remap-path-prefix") {
                let indent = &line[..line.find('"')
                    .unwrap_or(0)];

                format!("{}\"--remap-path-prefix\", \"{}=\"", indent, userprofile)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    fs::write(&config_file_path, updated).ok();
}

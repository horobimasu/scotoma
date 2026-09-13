use std::fs;
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::state::PROJECT_ROOT_DIR_PATH;

pub async fn start_build() -> impl IntoResponse {
    let root = PROJECT_ROOT_DIR_PATH
        .get()
        .unwrap();

    let encryptor_dir_path = root.join("encryptor");
    let decryptor_dir_path = root.join("decryptor");

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-pc-windows-msvc")
        .current_dir(&encryptor_dir_path)
        .creation_flags(0x08000000)
        .status();

    if status.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-pc-windows-msvc")
        .current_dir(&decryptor_dir_path)
        .creation_flags(0x08000000)
        .status();

    if status.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let builds_dir = root.join(".builds");
    if fs::create_dir_all(&builds_dir).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if let Ok(entries) = fs::read_dir(&builds_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with("latest-") {
                let new_name = name_str.replace("latest-", "");
                fs::rename(entry.path(), builds_dir.join(new_name)).ok();
            }
        }
    }

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let latest_dir = builds_dir.join(format!("latest-{}", time));
    if fs::create_dir_all(&latest_dir).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let encryptor_original = encryptor_dir_path
            .join("target\\x86_64-pc-windows-msvc\\release\\encryptor.exe");

    let decryptor_original = decryptor_dir_path
        .join("target\\x86_64-pc-windows-msvc\\release\\decryptor.exe");

    if fs::rename(&encryptor_original, latest_dir.join("encryptor.exe")).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if fs::rename(&decryptor_original, latest_dir.join("decryptor.exe")).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

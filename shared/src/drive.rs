use std::path::PathBuf;
use std::io::Write;
use std::io::ErrorKind::StorageFull;
use std::fs::{self, OpenOptions};
use std::sync::atomic::{AtomicBool, Ordering};

use rand::Rng;
use rand::distributions::Alphanumeric;

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::WRITE_FILE_CHUNK_SIZE;

const FILE_NAME_LEN: usize = 10;
const MAX_FILE_SIZE: u64 = 3 * 1024 * 1024 * 1024;

pub fn get_drives() -> Vec<PathBuf> {
    let mut drives = Vec::new();

    for letter in b'A' ..=b'Z' {
        let drive = format!("{}:\\", letter as char);
        let path = PathBuf::from(&drive);

        if path.metadata().is_ok() {
            drives.push(path);
        }
    }

    drives
}

pub fn overwrite_drive_free_space(drive: &PathBuf, passes: usize) {
    let folder_name = {
        let mut rng = rand::thread_rng();

        (0 .. FILE_NAME_LEN)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect::<String>()
    };

    let folder_path = drive.join(folder_name);
    if fs::create_dir(&folder_path).is_err() {
        return;
    }

    for pass in 0 .. passes {
        let drive_full = AtomicBool::new(false);

        (0 .. num_cpus::get()).into_par_iter().for_each(|_| {
            let mut rng = rand::thread_rng();
            let mut buffer = vec![0u8; WRITE_FILE_CHUNK_SIZE];

            while !drive_full.load(Ordering::Relaxed) {
                let file_name = (0 .. FILE_NAME_LEN)
                    .map(|_| rng.sample(Alphanumeric) as char)
                    .collect::<String>();

                let file_path = folder_path.join(&file_name);

                let file_open = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&file_path);

                let mut file = match file_open {
                    Ok(file) => file,
                    Err(_) => {
                        drive_full.store(true, Ordering::Relaxed);
                        break;
                    }
                };

                let mut bytes_written = 0;
                let mut disk_full = false;

                while bytes_written < MAX_FILE_SIZE {
                    let chunk_size = (WRITE_FILE_CHUNK_SIZE as u64)
                        .min(MAX_FILE_SIZE - bytes_written) as usize;

                    match pass % 3 {
                        0 => buffer[..chunk_size].fill(0x00),
                        1 => buffer[..chunk_size].fill(0xFF),
                        _ => rng.fill(&mut buffer[..chunk_size])
                    }

                    match file.write_all(&buffer[..chunk_size]) {
                        Ok(_) => bytes_written += chunk_size as u64,
                        Err(err) => {
                            if err.kind() == StorageFull {
                                disk_full = true;
                                drive_full.store(true, Ordering::Relaxed);
                            }

                            break;
                        }
                    }
                }

                file.sync_data().ok();

                if disk_full {
                    break;
                }
            }
        });
    }

    fs::remove_dir_all(&folder_path).ok();
}

#[test]
fn test_get_drives() {
    for drive in get_drives() {
        println!("drive - {}", drive.display());
    }
}

#[test]
fn test_overwrite_drive_free_space() {
    const DRIVE_TO_OVERWRITE: &str = "D:\\";
    const PASSES: usize = 3;

    let drive_path = PathBuf::from(DRIVE_TO_OVERWRITE);

    overwrite_drive_free_space(&drive_path, PASSES);
}

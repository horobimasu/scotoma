use std::error::Error;

use rand::Rng;

use chacha20poly1305::{Error as XError, XChaCha20Poly1305, Key, XNonce};
use chacha20poly1305::aead::{Aead, KeyInit};

pub const TAG_LEN: usize = 16;

const NONCE_LEN: usize = 24;

pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, XError> {
    let mut nonce_buffer = [0u8; NONCE_LEN];
    rand::thread_rng().fill(&mut nonce_buffer);

    let chacha_key = Key::from_slice(&key);
    let chacha_nonce = XNonce::from_slice(&nonce_buffer);

    let chacha_engine = XChaCha20Poly1305::new(&chacha_key);
    let chacha_encrypted = chacha_engine.encrypt(&chacha_nonce, data)?;

    let mut result = nonce_buffer.to_vec();
    result.extend_from_slice(&chacha_encrypted);

    Ok(result)
}

pub fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    if encrypted_data.len() <= NONCE_LEN {
        return Err("data is too short".into());
    }

    let (nonce, encrypted) = encrypted_data.split_at(NONCE_LEN);

    let chacha_key = Key::from_slice(&key);
    let chacha_nonce = XNonce::from_slice(&nonce);

    let chacha_engine = XChaCha20Poly1305::new(&chacha_key);

    let result = chacha_engine
        .decrypt(&chacha_nonce, encrypted)
        .map_err(|err| err.to_string())?;

    Ok(result)
}

#[test]
fn test_crypt_data() {
    const ORIGINAL_DATA: &str = "the original data";
    const CRYPT_KEY: &str = "the key for encryption";

    let hashed_key = crate::hash::hash_argon2id(CRYPT_KEY.as_bytes());

    let encrypted = match encrypt_data(ORIGINAL_DATA.as_bytes(), &hashed_key) {
        Ok(data) => data,
        Err(err) => panic!("failed to encrypt - {}", err)
    };

    let decrypted = match decrypt_data(&encrypted, &hashed_key) {
        Ok(data) => data,
        Err(err) => panic!("failed to decrypt - {}", err)
    };

    assert_eq!(decrypted, ORIGINAL_DATA.as_bytes());
}

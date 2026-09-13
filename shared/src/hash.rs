use argon2::Argon2;

pub fn hash_argon2id(data: &[u8]) -> Vec<u8> {
    let mut hashed_data_buffer = [0u8; 32];

    Argon2::default()
        .hash_password_into(data, data, &mut hashed_data_buffer)
        .unwrap();

    hashed_data_buffer.to_vec()
}

pub fn hash_blake3(data: &[u8]) -> Vec<u8> {
    blake3::hash(data)
        .as_bytes()
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA_TO_HASH: &str = "the data to hash for testing";

    #[test]
    fn test_hash_argon2id() {
        let hash1 = hash_argon2id(DATA_TO_HASH.as_bytes());
        let hash2 = hash_argon2id(DATA_TO_HASH.as_bytes());

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_blake3() {
        let hash1 = hash_blake3(DATA_TO_HASH.as_bytes());
        let hash2 = hash_blake3(DATA_TO_HASH.as_bytes());

        assert_eq!(hash1, hash2);
    }
}

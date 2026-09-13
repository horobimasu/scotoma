use std::error::Error;

use rand::rngs::OsRng;

use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

use pqc_kyber::{KYBER_SECRETKEYBYTES, KYBER_CIPHERTEXTBYTES, KYBER_PUBLICKEYBYTES};
use pqc_kyber::{PublicKey as KyberPublic, SecretKey as KyberSecret};

use zeroize::Zeroize;

use crate::crypt::*;

const X25519_PRIVATE_LEN: usize = 32;
const X25519_PUBLIC_LEN: usize = 32;

const DERIVE_CRYPT_KEY_CONTEXT: &str = "ckctx";

pub struct PrivateKeys {
    pub x25519: StaticSecret,
    pub kyber: KyberSecret
}

pub struct PublicKeys {
    pub x25519: PublicKey,
    pub kyber: KyberPublic
}

pub fn generate_key_pairs() -> Result<(PrivateKeys, PublicKeys), Box<dyn Error>> {
    let x25519_private = StaticSecret::random_from_rng(OsRng);
    let x25519_prublic = PublicKey::from(&x25519_private);

    let key_pair = pqc_kyber::keypair(&mut OsRng)?;

    let private_keys = PrivateKeys {
        x25519: x25519_private,
        kyber: key_pair.secret
    };

    let public_keys = PublicKeys {
        x25519: x25519_prublic,
        kyber: key_pair.public
    };

    Ok((private_keys, public_keys))
}

pub fn export_key_pairs(
    private_keys: &PrivateKeys,
    public_keys: &PublicKeys
) -> (Vec<u8>, Vec<u8>) {
    let mut private = Vec::new();
    private.extend_from_slice(&private_keys.x25519.to_bytes());
    private.extend_from_slice(&private_keys.kyber);

    let mut public = Vec::new();
    public.extend_from_slice(public_keys.x25519.as_bytes());
    public.extend_from_slice(&public_keys.kyber);

    (private, public)
}

pub fn import_key_pairs(
    private_keys: Option<&[u8]>,
    public_keys: Option<&[u8]>
) -> Result<(Option<PrivateKeys>, Option<PublicKeys>), Box<dyn Error>> {
    let private = if let Some(private_keys) = private_keys {
        if private_keys.len() < X25519_PRIVATE_LEN + KYBER_SECRETKEYBYTES {
            return Err("private keys too short".into());
        }

        let x25519 = StaticSecret::from(
            <[u8; X25519_PRIVATE_LEN]>::try_from(&private_keys[..X25519_PRIVATE_LEN])?
        );

        let mut kyber = [0u8; KYBER_SECRETKEYBYTES];
        kyber.copy_from_slice(&private_keys[X25519_PRIVATE_LEN .. X25519_PRIVATE_LEN + KYBER_SECRETKEYBYTES]);

        Some(PrivateKeys { x25519, kyber })
    } else {
        None
    };

    let public = if let Some(public_keys) = public_keys {
        if public_keys.len() < X25519_PUBLIC_LEN + KYBER_PUBLICKEYBYTES {
            return Err("public keys too short".into());
        }

        let x25519 = x25519_dalek::PublicKey::from(
            <[u8; X25519_PUBLIC_LEN]>::try_from(&public_keys[..X25519_PUBLIC_LEN])?
        );

        let mut kyber = [0u8; KYBER_PUBLICKEYBYTES];
        kyber.copy_from_slice(&public_keys[X25519_PUBLIC_LEN .. X25519_PUBLIC_LEN + KYBER_PUBLICKEYBYTES]);

        Some(PublicKeys { x25519, kyber })
    } else {
        None
    };

    Ok((private, public))
}

pub fn encrypt_key(key: &mut [u8], public_keys: &PublicKeys) -> Result<Vec<u8>, Box<dyn Error>> {
    let ephemeral_private = EphemeralSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_private);

    let shared_secret = ephemeral_private.diffie_hellman(&public_keys.x25519);

    let (kyber_cipher, kyber_secret) = pqc_kyber::encapsulate(&public_keys.kyber, &mut OsRng)?;

    let mut joined_secrets = Vec::new();
    joined_secrets.extend_from_slice(shared_secret.as_bytes());
    joined_secrets.extend_from_slice(&kyber_secret.to_vec());

    let mut crypt_key = blake3::derive_key(DERIVE_CRYPT_KEY_CONTEXT, &joined_secrets);
    joined_secrets.zeroize();

    let encrypted_key = encrypt_data(key, &crypt_key)
        .map_err(|_| "failed to encrypt")?;

    crypt_key.zeroize();
    key.zeroize();

    let mut encrypted_key_blob = Vec::new();
    encrypted_key_blob.extend_from_slice(ephemeral_public.as_bytes());
    encrypted_key_blob.extend_from_slice(&kyber_cipher);
    encrypted_key_blob.extend_from_slice(&encrypted_key);

    Ok(encrypted_key_blob)
}

pub fn decrypt_key(
    encrypted_key: &[u8],
    private_keys: &PrivateKeys
) -> Result<Vec<u8>, Box<dyn Error>> {
    let ephemeral_public_bytes = &encrypted_key[..X25519_PUBLIC_LEN];
    let kyber_ciphered_bytes = &encrypted_key[X25519_PUBLIC_LEN .. X25519_PUBLIC_LEN + KYBER_CIPHERTEXTBYTES];
    let encrypted_key = &encrypted_key[X25519_PUBLIC_LEN + KYBER_CIPHERTEXTBYTES..];

    let ephemeral_public = PublicKey::from(
        <[u8; X25519_PUBLIC_LEN]>::try_from(ephemeral_public_bytes)?
    );

    let shared_secret = private_keys.x25519
        .diffie_hellman(&ephemeral_public);

    if kyber_ciphered_bytes.len() != pqc_kyber::KYBER_CIPHERTEXTBYTES {
        return Err("invalid kyber cipher length".into());
    }

    let kyber_secret = pqc_kyber::decapsulate(kyber_ciphered_bytes, &private_keys.kyber)?;

    let mut joined_secrets = Vec::new();
    joined_secrets.extend_from_slice(shared_secret.as_bytes());
    joined_secrets.extend_from_slice(&kyber_secret);

    let mut crypt_key = blake3::derive_key(DERIVE_CRYPT_KEY_CONTEXT, &joined_secrets);
    joined_secrets.zeroize();

    let decrypted_key = decrypt_data(encrypted_key, &crypt_key)
        .map_err(|_| "failed to decrypt")?;

    crypt_key.zeroize();

    Ok(decrypted_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_import_key_pairs() {
        let (private_keys, public_keys) = generate_key_pairs()
            .unwrap();

        let (private_keys_bytes, public_keys_bytes) = export_key_pairs(&private_keys, &public_keys);

        let (imported_private_keys, imported_public_keys) = match import_key_pairs(
            Some(&private_keys_bytes),
            Some(&public_keys_bytes)
        ) {
            Ok((private, public)) => (private, public),
            Err(err) => panic!("failed to import key pair - {}", err)
        };

        let (reimported_private_bytes, reimported_public_bytes) = export_key_pairs(
            &imported_private_keys.unwrap(),
            &imported_public_keys.unwrap()
        );

        assert_eq!(private_keys_bytes, reimported_private_bytes);
        assert_eq!(public_keys_bytes, reimported_public_bytes);
    }

    #[test]
    fn test_crypt_key() {
        use rand::Rng;

        let mut main_key_buffer = [0u8; crate::KEY_LENGTH];
        rand::thread_rng().fill(&mut main_key_buffer);

        let original_key = main_key_buffer;

        let (private_keys, public_keys) = generate_key_pairs()
            .unwrap();

        let encrypted_key = match encrypt_key(&mut main_key_buffer, &public_keys) {
            Ok(key) => key,
            Err(err) => panic!("failed to encrypt - {}", err)
        };

        let decrypted_key = match decrypt_key(&encrypted_key, &private_keys) {
            Ok(key) => key,
            Err(err) => panic!("failed to decrypt - {}", err)
        };

        assert_eq!(original_key, decrypted_key.as_slice());
    }
}

use aes_gcm::aead::{AeadInPlace, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;

fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), password.as_bytes());
    let mut key = [0u8; KEY_SIZE];
    hkdf.expand(&[], &mut key).unwrap();
    *Key::<Aes256Gcm>::from_slice(&key)
}

pub fn encrypt_data(data: &[u8], password: &str) -> io::Result<Vec<u8>> {
    let salt = generate_random_bytes(16);
    let nonce = generate_random_bytes(NONCE_SIZE);
    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(&key);

    let mut buffer = data.to_vec();
        cipher
        .encrypt_in_place(Nonce::from_slice(&nonce), b"", &mut buffer)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Encryption failed"))?;

    let mut result = Vec::new();
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&buffer);
    Ok(result)
}

pub fn decrypt_data(data: &[u8], password: &str) -> io::Result<Vec<u8>> {
    let (salt, rest) = data.split_at(16);
    let (nonce, enc_data) = rest.split_at(NONCE_SIZE);
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(&key);

    let mut buffer = enc_data.to_vec();
    cipher
        .decrypt_in_place(Nonce::from_slice(nonce), b"", &mut buffer)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Decryption failed"))?;
    Ok(buffer)
}

fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

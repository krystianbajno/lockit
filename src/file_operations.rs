use std::fs;
use std::path::{Path, PathBuf};
use hex;

include!(concat!(env!("OUT_DIR"), "/default_settings.rs"));

pub fn process_file_with_flags(file_path: &Path, password: &str, encrypt: bool, encrypt_filenames: bool) -> Option<PathBuf> {
    if encrypt {
        compress_and_encrypt_file(file_path, password, encrypt_filenames)
    } else {
        decompress_and_decrypt_file(file_path, password, encrypt_filenames)
    }
}

pub fn process_directory_with_flags(
    directory_path: &Path,
    password: &str,
    encrypt: bool,
    encrypt_filenames: bool,
) {
    for entry in fs::read_dir(directory_path).unwrap() {
        let entry_path = entry.unwrap().path();

        if entry_path.is_file() {
            process_file_with_flags(&entry_path, password, encrypt, encrypt_filenames);
        } else if entry_path.is_dir() {
            process_directory_with_flags(&entry_path, password, encrypt, encrypt_filenames);
        }
    }
}

fn encrypt_filename(filename: &str, password: &str) -> Option<String> {
    let encrypted_data = super::crypto::encrypt_data(filename.as_bytes(), password).ok()?;
    Some(hex::encode(encrypted_data))
}

fn decrypt_filename(hex_encoded: &str, password: &str) -> Option<String> {
    let encrypted_data = hex::decode(hex_encoded).ok()?;
    let decrypted_data = super::crypto::decrypt_data(&encrypted_data, password).ok()?;
    String::from_utf8(decrypted_data).ok()
}

fn get_new_filename(file_path: &Path, password: &str, encrypt: bool, encrypt_filenames: bool) -> Option<String> {
    if encrypt_filenames {
        if encrypt {
            let filename = file_path.file_name()?.to_str()?;
            encrypt_filename(filename, password)
        } else {
            let filename = file_path.file_stem()?.to_str()?;
            decrypt_filename(filename, password)
        }
    } else {
        Some(file_path.file_name()?.to_str()?.to_string())
    }
}

fn compress_and_encrypt_file(file_path: &Path, password: &str, encrypt_filenames: bool) -> Option<PathBuf> {
    let file_data = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path.display(), e);
            return None;
        }
    };

    let compressed_data = match super::compression::compress_data(&file_data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error compressing file {}: {}", file_path.display(), e);
            return None;
        }
    };

    let encrypted_data = match super::crypto::encrypt_data(&compressed_data, password) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error encrypting file {}: {}", file_path.display(), e);
            return None;
        }
    };

    let new_filename = get_new_filename(file_path, password, true, encrypt_filenames)?;
    let new_file_path = file_path.with_file_name(format!("{}.{}", new_filename, CUSTOM_EXTENSION));

    if fs::write(&new_file_path, &encrypted_data).is_err() {
        eprintln!("Error writing encrypted file {}", new_file_path.display());
        return None;
    }

    fs::remove_file(file_path).unwrap();
    Some(new_file_path)
}

fn decompress_and_decrypt_file(file_path: &Path, password: &str, encrypt_filenames: bool) -> Option<PathBuf> {
    if file_path.extension()?.to_str()? != CUSTOM_EXTENSION {
        eprintln!("Skipping file with unsupported extension: {}", file_path.display());
        return None;
    }

    let encrypted_data = match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading encrypted file {}: {}", file_path.display(), e);
            return None;
        }
    };

    let decrypted_data = match super::crypto::decrypt_data(&encrypted_data, password) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error decrypting file {}: {}", file_path.display(), e);
            return None;
        }
    };

    let decompressed_data = match super::compression::decompress_data(&decrypted_data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error decompressing file {}: {}", file_path.display(), e);
            return None;
        }
    };

    let new_filename = get_new_filename(file_path, password, false, encrypt_filenames)?;
    let output_path = file_path.with_file_name(new_filename);

    if fs::write(&output_path, &decompressed_data).is_err() {
        eprintln!(
            "Error writing decompressed file {}",
            output_path.display()
        );
        return None;
    }

    fs::remove_file(file_path).unwrap();
    Some(output_path)
}

use std::fs;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};
use tar::Archive;
use hex;

include!(concat!(env!("OUT_DIR"), "/default_settings.rs"));

pub fn process_file_with_flags(
    file_path: &Path, 
    password: &str, 
    encrypt: bool, 
    encrypt_filenames: bool,
) -> Option<PathBuf> {
    let dir_lockit_extension = format!("{}.{}", CUSTOM_DIRECTORY_EXTENSION, CUSTOM_EXTENSION);

    if !encrypt && file_path.file_name()?.to_str()?.ends_with(&dir_lockit_extension) {
        return decrypt_and_extract_dir_lockit(file_path, password, encrypt_filenames);
    }

    match encrypt {
        true => compress_and_encrypt_file(file_path, password, encrypt_filenames),
        false => decompress_and_decrypt_file(file_path, password, encrypt_filenames),
    }
}

pub fn process_directory_with_flags(
    directory_path: &Path,
    password: &str,
    encrypt: bool,
    encrypt_filenames: bool,
    dir_mode: bool,
) -> Option<()> {
    if dir_mode && encrypt {
        if let Some(encrypted_tar_data) = create_compress_encrypt_tar(directory_path, password) {
            let new_filename = get_new_filename(directory_path, password, true, encrypt_filenames)?;
            let tar_filename = directory_path.with_file_name(format!("{}.{}.{}", new_filename, CUSTOM_DIRECTORY_EXTENSION, CUSTOM_EXTENSION));

            if fs::write(&tar_filename, &encrypted_tar_data).is_err() {
                eprintln!("Failed to write encrypted tar file {}", tar_filename.display());
            } else if fs::remove_dir_all(directory_path).is_err() {
                eprintln!("Failed to remove original directory {}", directory_path.display());
            }
        }
    } else {
        for entry in fs::read_dir(directory_path).unwrap() {
            let entry_path = entry.unwrap().path();
            if entry_path.is_file() {
                process_file_with_flags(&entry_path, password, encrypt, encrypt_filenames);
            } else if entry_path.is_dir() {
                process_directory_with_flags(&entry_path, password, encrypt, encrypt_filenames, dir_mode)?;
            }
        }
    }
    Some(())
}

fn create_compress_encrypt_tar(directory_path: &Path, password: &str) -> Option<Vec<u8>> {
    let mut tar_data = Vec::new();
    {
        let mut tar_builder = tar::Builder::new(&mut tar_data);
        if tar_builder.append_dir_all(".", directory_path).is_err() || tar_builder.finish().is_err() {
            eprintln!("Failed to create or finalize tar archive");
            return None;
        }
    }

    match super::compression::compress_data(&tar_data)
        .and_then(|data| super::crypto::encrypt_data(&data, password)) {
            Ok(encrypted_data) => Some(encrypted_data),
            Err(_) => {
                eprintln!("Failed to compress or encrypt tar archive");
                None
            }
    }
}

fn decrypt_and_extract_dir_lockit(file_path: &Path, password: &str, encrypt_filenames: bool) -> Option<PathBuf> {
    if let Some(decrypted_data) = decompress_and_decrypt_tar(file_path, password) {
        let encrypted_dir_name = file_path.with_extension("").file_stem()?.to_string_lossy().to_string();
        let decrypted_dir_name = if encrypt_filenames {
            decrypt_filename(&encrypted_dir_name, password)?
        } else {
            encrypted_dir_name
        };
        let extraction_path = Path::new(&decrypted_dir_name);

        if extract_tar_archive(&decrypted_data, extraction_path).is_err() {
            eprintln!("Failed to extract tar archive");
            return None;
        } else if fs::remove_file(file_path).is_err() {
            eprintln!("Failed to remove encrypted tar file {}", file_path.display());
        }

        return Some(extraction_path.to_path_buf());
    }

    None
}

fn decompress_and_decrypt_tar(tar_file_path: &Path, password: &str) -> Option<Vec<u8>> {
    let encrypted_data = match fs::read(tar_file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading encrypted tar file {}: {}", tar_file_path.display(), e);
            return None;
        }
    };

    let decrypted_data = match super::crypto::decrypt_data(&encrypted_data, password) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error decrypting tar file {}: {}", tar_file_path.display(), e);
            return None;
        }
    };

    let decompressed_data = match super::compression::decompress_data(&decrypted_data) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error decompressing tar file {}: {}", tar_file_path.display(), e);
            return None;
        }
    };

    Some(decompressed_data)
}

fn extract_tar_archive(tar_data: &[u8], output_dir: &Path) -> io::Result<()> {
    let cursor = Cursor::new(tar_data);
    let mut archive = Archive::new(cursor);
    archive.unpack(output_dir)
}

fn encrypt_filename(filename: &str, password: &str) -> Option<String> {
    super::crypto::encrypt_data(filename.as_bytes(), password)
        .ok()
        .map(hex::encode)
}

fn decrypt_filename(hex_encoded: &str, password: &str) -> Option<String> {
    hex::decode(hex_encoded).ok()
        .and_then(|encrypted_data| super::crypto::decrypt_data(&encrypted_data, password).ok())
        .and_then(|decrypted_data| String::from_utf8(decrypted_data).ok())
}

fn get_new_filename(file_path: &Path, password: &str, encrypt: bool, encrypt_filenames: bool) -> Option<String> {
    if encrypt_filenames {
        match encrypt {
            true => file_path.file_name()?.to_str().and_then(|name| encrypt_filename(name, password)),
            false => file_path.file_stem()?.to_str().and_then(|name| decrypt_filename(name, password)),
        }
    } else {
        file_path.file_name()?.to_str().map(String::from)
    }
}

fn compress_and_encrypt_file(file_path: &Path, password: &str, encrypt_filenames: bool) -> Option<PathBuf> {
    let file_data = fs::read(file_path).ok()?;
    let compressed_data = super::compression::compress_data(&file_data).ok()?;
    let encrypted_data = super::crypto::encrypt_data(&compressed_data, password).ok()?;

    let new_filename = get_new_filename(file_path, password, true, encrypt_filenames)?;
    let new_file_path = file_path.with_file_name(format!("{}.{}", new_filename, CUSTOM_EXTENSION));

    if fs::write(&new_file_path, &encrypted_data).is_ok() {
        fs::remove_file(file_path).ok()?;
        Some(new_file_path)
    } else {
        eprintln!("Error writing encrypted file {}", new_file_path.display());
        None
    }
}

fn decompress_and_decrypt_file(file_path: &Path, password: &str, encrypt_filenames: bool) -> Option<PathBuf> {
    if file_path.extension()?.to_str()? != CUSTOM_EXTENSION {
        eprintln!("Skipping file with unsupported extension: {}", file_path.display());
        return None;
    }

    let encrypted_data = fs::read(file_path).ok()?;
    let decrypted_data = super::crypto::decrypt_data(&encrypted_data, password).ok()?;
    let decompressed_data = super::compression::decompress_data(&decrypted_data).ok()?;

    let new_filename = get_new_filename(file_path, password, false, encrypt_filenames)?;
    let output_path = file_path.with_file_name(new_filename);

    if fs::write(&output_path, &decompressed_data).is_ok() {
        fs::remove_file(file_path).ok()?;
        Some(output_path)
    } else {
        eprintln!("Error writing decompressed file {}", output_path.display());
        None
    }
}

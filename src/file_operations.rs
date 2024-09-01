use std::fs::{self, OpenOptions};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tar::Archive;
use hex;

include!(concat!(env!("OUT_DIR"), "/default_settings.rs"));

pub fn process_file_with_flags(
    file_path: &Path, 
    password: &str, 
    encrypt: bool, 
    encrypt_filenames: bool,
    skip_dod: bool,
) -> Option<PathBuf> {
    let dir_lockit_extension = format!("{}.{}", CUSTOM_DIRECTORY_EXTENSION, CUSTOM_EXTENSION);

    if !encrypt && file_path.file_name()?.to_str()?.ends_with(&dir_lockit_extension) {
        return decrypt_and_extract_dir_lockit(file_path, password, encrypt_filenames, skip_dod);
    }

    match encrypt {
        true => compress_and_encrypt_file(file_path, password, encrypt_filenames, skip_dod),
        false => decompress_and_decrypt_file(file_path, password, encrypt_filenames, skip_dod),
    }
}

pub fn process_directory_with_flags(
    directory_path: &Path,
    password: &str,
    encrypt: bool,
    encrypt_filenames: bool,
    dir_mode: bool,
    skip_dod: bool,
) -> Option<()> {
    if dir_mode && encrypt {
        if let Some(encrypted_tar_data) = create_compress_encrypt_tar(directory_path, password) {
            let new_filename = get_new_filename(directory_path, password, true, encrypt_filenames)?;
            let tar_filename = directory_path.with_file_name(format!("{}.{}.{}", new_filename, CUSTOM_DIRECTORY_EXTENSION, CUSTOM_EXTENSION));

            if fs::write(&tar_filename, &encrypted_tar_data).is_err() {
                eprintln!("Failed to write encrypted tar file {}", tar_filename.display());
            } else if secure_delete_directory(directory_path, skip_dod).is_err() {
                eprintln!("Failed to securely delete original directory {}", directory_path.display());
            }
        }
    } else {
        for entry in fs::read_dir(directory_path).unwrap() {
            let entry_path = entry.unwrap().path();
            if entry_path.is_file() {
                process_file_with_flags(&entry_path, password, encrypt, encrypt_filenames, skip_dod);
            } else if entry_path.is_dir() {
                process_directory_with_flags(&entry_path, password, encrypt, encrypt_filenames, dir_mode, skip_dod)?;
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

fn decrypt_and_extract_dir_lockit(file_path: &Path, password: &str, encrypt_filenames: bool, skip_dod: bool) -> Option<PathBuf> {
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
        } else if secure_delete(file_path, skip_dod).is_err() {
            eprintln!("Failed to securely delete encrypted tar file {}", file_path.display());
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

fn compress_and_encrypt_file(file_path: &Path, password: &str, encrypt_filenames: bool, skip_dod: bool) -> Option<PathBuf> {
    let file_data = fs::read(file_path).ok()?;
    let compressed_data = super::compression::compress_data(&file_data).ok()?;
    let encrypted_data = super::crypto::encrypt_data(&compressed_data, password).ok()?;

    let new_filename = get_new_filename(file_path, password, true, encrypt_filenames)?;
    let new_file_path = file_path.with_file_name(format!("{}.{}", new_filename, CUSTOM_EXTENSION));

    if fs::write(&new_file_path, &encrypted_data).is_ok() {
        if let Err(e) = secure_delete(file_path, skip_dod) {
            eprintln!("Error securely deleting file {}: {}", file_path.display(), e);
        }
        Some(new_file_path)
    } else {
        eprintln!("Error writing encrypted file {}", new_file_path.display());
        None
    }
}

fn decompress_and_decrypt_file(file_path: &Path, password: &str, encrypt_filenames: bool, skip_dod: bool) -> Option<PathBuf> {
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
        if let Err(e) = secure_delete(file_path, skip_dod) {
            eprintln!("Error securely deleting file {}: {}", file_path.display(), e);
        }
        Some(output_path)
    } else {
        eprintln!("Error writing decompressed file {}", output_path.display());
        None
    }
}

pub fn secure_delete(path: &Path, skip_dod: bool) -> io::Result<()> {
    if path.exists() {
        if skip_dod {
            fs::remove_file(path)?;
            return Ok(());
        }

        let metadata = fs::metadata(&path)?;
        let file_size = metadata.len();

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(&path)?;

        fn verify_pass(file: &mut std::fs::File, expected: &[u8]) -> io::Result<bool> {
            let mut buffer = vec![0u8; expected.len()];
            file.seek(SeekFrom::Start(0))?;
            file.read_exact(&mut buffer)?;
            Ok(buffer == expected)
        }

        let pass1 = vec![0xFF; file_size as usize];
        file.write_all(&pass1)?;
        file.sync_all()?;

        file.seek(SeekFrom::Start(0))?;
        let pass2 = vec![0x00; file_size as usize];
        file.write_all(&pass2)?;
        file.sync_all()?;

        file.seek(SeekFrom::Start(0))?;
        let random_data: Vec<u8> = (0..file_size).map(|_| rand::random::<u8>()).collect();
        file.write_all(&random_data)?;
        file.sync_all()?;

        if !verify_pass(&mut file, &random_data)? {
            return Err(io::Error::new(io::ErrorKind::Other, "Verification failed at pass 3"));
        }

        drop(file);
        fs::remove_file(path)?;
    }

    Ok(())
}

pub fn secure_delete_directory(directory_path: &Path, skip_dod: bool) -> io::Result<()> {
    for entry in fs::read_dir(directory_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            secure_delete_directory(&path, skip_dod)?;
        } else {
            secure_delete(&path, skip_dod)?;
        }
    }

    fs::remove_dir(directory_path)
}

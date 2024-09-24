use std::{env, io, path::PathBuf};
use std::io::{Read, Write};

mod crypto;
mod compression;
mod file_operations;
mod pipe;

use file_operations::{secure_delete, secure_delete_directory};

include!(concat!(env!("OUT_DIR"), "/default_settings.rs"));

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&String::from("-h")) || args.contains(&String::from("--help")) {
        print_help();
        return;
    }
    
    let (mode, paths, encrypt_filenames_flag, self_destruct_flag, dir_mode, pipe_mode, skip_dod) = parse_mode_paths_and_flags(&args);

    let use_custom_password = args.contains(&String::from("-p"));

    let encrypt_filenames = encrypt_filenames_flag.unwrap_or(ENCRYPT_FILENAMES);

    let encrypt = is_encrypt_mode(&mode);

    if pipe_mode {
        if mode != "encrypt" && mode != "decrypt" {
            eprintln!("Error: --pipe mode can only be used with 'encrypt' or 'decrypt' modes.");
            std::process::exit(1);
        }
    }

    let password = get_password(use_custom_password);

    if pipe_mode {
        process_pipe_mode(&password, encrypt);
    } else {
        match mode.as_str() {
            "remove" | "delete" | "rm" | "del" => process_removal(paths, skip_dod),
            _ => process_paths(paths, &password, encrypt, encrypt_filenames, dir_mode, skip_dod),
        }

        if self_destruct_flag.unwrap_or(SELF_DESTRUCT_DEFAULT) {
            secure_self_destruct(skip_dod);
        }
    }
}

fn print_help() {
    println!(
        r#"Lockit - A Blazing Fast Encryption Suite

Usage:
    ./lockit <file> <file2> <dir1>             # Process files in default mode (encrypt/decrypt)
    ./lockit encrypt <file> <file2> <dir1>     # Encrypt specific files
    ./lockit encrypt <file> -p                 # Encrypt a file with a custom passphrase
    ./lockit encrypt <dir>                     # Encrypt all files in a directory
    ./lockit decrypt <dir/file>                # Decrypt a file or directory
    ./lockit encrypt <dir> --zipdir            # Compress, tar, and encrypt entire directories
    ./lockit remove/delete/rm/del <dir/file>   # Securely delete a file / directory
    ./lockit remove <file> <file2> --skip-dod  # Skip DoD overwrite passes
    ./lockit encrypt --pipe                    # Process as pipe
    ./lockit decrypt --pipe -p                 # Process as pipe, custom passphrase
    ./lockit --encrypt-filenames               # Encrypt file and directory names
    ./lockit --no-encrypt-filenames            # Keep file and directory names unchanged
    ./lockit --self-destruct                   # Remove Lockit after use
    ./lockit --no-self-destruct                # Retain Lockit after use
    ./lockit -h | --help                       # Show this help message

Examples:
    echo "Secret message" | ./lockit encrypt --pipe | ./lockit decrypt --pipe
    cat plaintext.txt | ./lockit encrypt --pipe > encrypted.enc
    nc -lvnp 9999 | ./lockit decrypt --pipe
    echo "This is a very secret message" | ./lockit encrypt --pipe | nc localhost 9999

Mechanism:
    Lockit compresses files using zstd and secures them with AES-256-GCM encryption.
    Provides secure file deletion following DoD 5220.22-M standard.

Installation:
    git clone https://github.com/krystianbajno/lockit
    cargo build --release

For more information, refer to the README.md file."#
    );
}


fn parse_mode_paths_and_flags(args: &[String]) -> (String, Vec<PathBuf>, Option<bool>, Option<bool>, bool, bool, bool) {
    let mut mode = DEFAULT_MODE.to_string();
    let mut paths = Vec::new();
    let mut encrypt_filenames_flag = None;
    let mut self_destruct_flag = None;
    let mut dir_mode = false;
    let mut pipe_mode = false;
    let mut skip_dod = SKIP_DOD_DEFAULT;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "encrypt" | "decrypt" | "remove" | "delete" | "rm" | "del" => mode = arg.clone(),
            "--encrypt-filenames" => encrypt_filenames_flag = Some(true),
            "--no-encrypt-filenames" => encrypt_filenames_flag = Some(false),
            "--self-destruct" => self_destruct_flag = Some(true),
            "--no-self-destruct" => self_destruct_flag = Some(false),
            "--zipdir" => dir_mode = true,
            "--pipe" => pipe_mode = true,
            "--skip-dod" => skip_dod = true,
            _ if !arg.starts_with('-') => paths.push(PathBuf::from(arg)),
            _ => {}
        }
    }

    if paths.is_empty() && !pipe_mode {
        paths.push(PathBuf::from("."));
    }

    (mode, paths, encrypt_filenames_flag, self_destruct_flag, dir_mode, pipe_mode, skip_dod)
}

fn is_encrypt_mode(mode: &str) -> bool {
    match mode {
        "encrypt" => true,
        "decrypt" => false,
        "delete" => false,
        "remove" => false,
        "rm" => false,
        "del" => false,
        _ => {
            println!("Invalid mode: {}. Defaulting to '{}'.", mode, DEFAULT_MODE);
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

fn process_pipe_mode(password: &str, encrypt: bool) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    let mut stdin_lock = stdin.lock();

    // Use a buffer to store the read data
    let mut buffer = [0u8; 1024]; // Buffer size of 1024 bytes

    loop {
        // Read from stdin into the buffer
        let bytes_read = match stdin_lock.read(&mut buffer) {
            Ok(0) => break, // EOF reached
            Ok(n) => n,     // Number of bytes read
            Err(e) => {
                eprintln!("Failed to read from stdin: {}", e);
                break;
            }
        };

        // Encrypt or decrypt the input data
        let processed_data = if encrypt {
            pipe::encrypt_data_via_pipe(&buffer[..bytes_read], password)
        } else {
            pipe::decrypt_data_via_pipe(&buffer[..bytes_read], password)
        };

        match processed_data {
            Ok(data) => {
                // Write the processed data to stdout
                if stdout_lock.write_all(&data).is_err() {
                    eprintln!("Failed to write to stdout");
                    break;
                }
                if stdout_lock.flush().is_err() {
                    eprintln!("Failed to flush stdout");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error processing data: {}", e);
                break;
            }
        }
    }
}


fn process_paths(paths: Vec<PathBuf>, password: &str, encrypt: bool, encrypt_filenames: bool, dir_mode: bool, skip_dod: bool) {
    for path in paths {
        if !path.exists() {
            debug_print!("Invalid path: {}", path.display());
            continue;
        }

        if path.is_file() {
            file_operations::process_file_with_flags(&path, password, encrypt, encrypt_filenames, skip_dod);
        } else if path.is_dir() {
            file_operations::process_directory_with_flags(&path, password, encrypt, encrypt_filenames, dir_mode, skip_dod);
        } else {
            debug_print!("Invalid path type: {}", path.display());
        }
    }
}

fn process_removal(paths: Vec<PathBuf>, skip_dod: bool) {
    for path in paths {
        if !path.exists() {
            debug_print!("Invalid path: {}", path.display());
            continue;
        }

        if path.is_file() {
            if let Err(e) = secure_delete(&path, skip_dod) {
                eprintln!("Error securely deleting file {}: {}", path.display(), e);
            }
        } else if path.is_dir() {
            if let Err(e) = secure_delete_directory(&path, skip_dod) {
                eprintln!("Error securely deleting directory {}: {}", path.display(), e);
            }
        } else {
            debug_print!("Invalid path type: {}", path.display());
        }
    }
}

fn secure_self_destruct(skip_dod: bool) {
    let current_exe = env::current_exe().expect("Failed to get the current executable path.");
    if let Err(e) = secure_delete(&current_exe, skip_dod) {
        eprintln!("Failed to securely delete the executable: {}", e);
    }
}

#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        eprintln!($($arg)*);
    }
}

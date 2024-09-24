use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    static_vcruntime::metabuild();

    let default_passphrase = "YourHardcodedPassphrase123!";
    let default_mode = "encrypt";
    let custom_extension: &str = "lockit";
    let custom_directory_extension: &str = "dir";
    let encrypt_filenames: bool = true; 
    let self_destruct_default = false;
    let skip_dod_default = false;

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir);

    fs::write(
        dest_path.join("default_settings.rs"),
        format!(
            "pub const DEFAULT_PASSPHRASE: &str = \"{}\";\n\
             pub const DEFAULT_MODE: &str = \"{}\";\n\
             pub const CUSTOM_EXTENSION: &str = \"{}\";\n\
             pub const CUSTOM_DIRECTORY_EXTENSION: &str = \"{}\";\n\
             pub const ENCRYPT_FILENAMES: bool = {};\n\
             pub const SELF_DESTRUCT_DEFAULT: bool = {};\n\
             pub const SKIP_DOD_DEFAULT: bool = {};",
            default_passphrase, default_mode, custom_extension, custom_directory_extension, encrypt_filenames, self_destruct_default, skip_dod_default
        ),
    )
    .unwrap();
}

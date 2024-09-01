# Lockit
Lock it, unlock it.

⚡ Blazing fast file encryption suite.

```bash
./lockit <file> <file2> <file3>            # Process files in default mode (encrypt/decrypt)
./lockit encrypt <file> <file2> <file3>    # Encrypt specific files
./lockit encrypt <file> -p                 # Encrypt a file with a custom password
./lockit encrypt <dir>                     # Encrypt all files in a directory
./lockit decrypt <dir/file>                # Decrypt a file or directory
./lockit encrypt <dir> --dir               # Compress, tar, and encrypt entire directories
./lockit remove/delete/rm/del <dir/file>   # Securely delete a file / directory.
./lockit --encrypt-filenames               # Encrypt file names
./lockit --no-encrypt-filenames            # Keep file names unchanged
./lockit --self-destruct                   # Remove Lockit after use
./lockit --no-self-destruct                # Retain Lockit after use

```

## Mechanism
Lockit compresses files using zstd and secures them with AES-256-GCM encryption. It also provides secure file deletion that follows DoD 5220.22-M standard.

## Customizing Default Settings
To change default settings, simply modify `build.rs`.








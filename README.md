# Lockit
Lock it, unlock it.

⚡ Blazing fast encryption suite.

```bash
./lockit <file> <file2> <dir1>             # Process files in default mode (encrypt/decrypt)
./lockit encrypt <file> <file2> <dir1>     # Encrypt specific files
./lockit encrypt <file> -p                 # Encrypt a file with a custom passphrase.
./lockit encrypt <dir>                     # Encrypt all files in a directory
./lockit decrypt <dir/file>                # Decrypt a file or directory
./lockit encrypt <dir> --dir               # Compress, tar, and encrypt entire directories
./lockit remove/delete/rm/del <dir/file>   # Securely delete a file / directory.
./lockit encrypt --pipe                    # Process as pipe.
./lockit decrypt --pipe -p                 # Process as pipe, custom passphrase.
./lockit --encrypt-filenames               # Encrypt file names
./lockit --no-encrypt-filenames            # Keep file names unchanged
./lockit --self-destruct                   # Remove Lockit after use
./lockit --no-self-destruct                # Retain Lockit after use
```

## Pipe
If `--pipe` is specified, the program processes the input from stdin and outputs to stdout instead of handling files or directories.

```bash
echo "Secret message" | ./lockit encrypt --pipe | ./lockit decrypt --pipe
```

## Mechanism
Lockit compresses files using zstd and secures them with AES-256-GCM encryption. It also provides secure file deletion that follows DoD 5220.22-M standard.

## Installation
```bash
git clone https://github.com/krystianbajno/lockit
cargo build --release
```

## Customizing Default Settings
To change default settings, simply modify `build.rs`.
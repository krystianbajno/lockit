# Lockit
Lock it, unlock it.

Blazing fast file encryption suite.

```bash
./lockit <file> <file2> <file3> # process files default mode (encrypt, decrypt)
./lockit encrypt <file> <file2> <file3> # encrypt files
./lockit encrypt <file> -p # use custom password
./lockit encrypt <dir> # encrypt files in directories
./lockit decrypt <dir/file> # decrypt files / directories
./lockit encrypt <dir> --dir # encrypt directories, but first tar contents and then compress and encrypt the tar file
./lockit --encrypt-filenames # encrypt filenames
./lockit --no-encrypt-filenames # do not encrypt filenames
./lockit --self-destruct # remove lockit after use
./lockit --no-self-destruct # do not remove lockit after use
```

## Mechanism
Files are zstd compressed and aes256gcm encrypted.

## Change default settings
Modify `build.rs`
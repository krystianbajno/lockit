# lockit
1.
- aes256gcm
- recursive
- hkdf passphrase
- zstd

2.
- from url mode
- pub key encrypts sym key, saves to file
- saves to url


3.
- data is sent to telegram
- operator has master key
- master key allows to change state
- state is saved in a json file
- encryption data is encrypted authentication is encryption with master key
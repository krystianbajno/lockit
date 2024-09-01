use std::io;

pub fn encrypt_data_via_pipe(input: &[u8], password: &str) -> Result<Vec<u8>, io::Error> {
    let compressed = super::compression::compress_data(input).unwrap();
    super::crypto::encrypt_data(&compressed, password).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

pub fn decrypt_data_via_pipe(input: &[u8], password: &str) -> Result<Vec<u8>, io::Error> {
    let decrypted = super::crypto::decrypt_data(input, password).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()));
    super::compression::decompress_data(&decrypted.unwrap())
}

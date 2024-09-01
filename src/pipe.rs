use std::io;

pub fn encrypt_data_via_pipe(input: &[u8], password: &str) -> Result<Vec<u8>, io::Error> {
    super::crypto::encrypt_data(input, password).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

pub fn decrypt_data_via_pipe(input: &[u8], password: &str) -> Result<Vec<u8>, io::Error> {
    super::crypto::decrypt_data(input, password).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

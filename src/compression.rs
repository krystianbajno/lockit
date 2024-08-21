use std::io;
use zstd::stream::{encode_all, decode_all};

pub fn compress_data(data: &[u8]) -> io::Result<Vec<u8>> {
    encode_all(data, 0)
}

pub fn decompress_data(data: &[u8]) -> io::Result<Vec<u8>> {
    decode_all(data)
}

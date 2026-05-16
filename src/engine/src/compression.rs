pub fn compress_fast(data: &str) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(data.as_bytes(), 3)
}

pub fn compress_best(data: &str) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(data.as_bytes(), 19)
}

pub fn decompress(compressed: &[u8]) -> Result<String, std::io::Error> {
    let decompressed = zstd::decode_all(compressed)?;
    String::from_utf8(decompressed)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

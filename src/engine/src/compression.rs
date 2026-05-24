pub fn compress_fast(data: &str) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(data.as_bytes(), 11)
}

pub fn compress_best(data: &str) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(data.as_bytes(), 19)
}

pub fn decompress(compressed: &[u8]) -> Result<String, std::io::Error> {
    let decompressed = zstd::decode_all(compressed)?;
    String::from_utf8(decompressed)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str =
        "Hello, world! This is a sample string for testing compression and decompression.";

    #[test]
    fn compress_fast_decompress_roundtrip() {
        let compressed = compress_fast(SAMPLE).expect("compress_fast failed");
        let decompressed = decompress(&compressed).expect("decompress failed");
        assert_eq!(decompressed, SAMPLE);
    }

    #[test]
    fn compress_best_decompress_roundtrip() {
        let compressed = compress_best(SAMPLE).expect("compress_best failed");
        let decompressed = decompress(&compressed).expect("decompress failed");
        assert_eq!(decompressed, SAMPLE);
    }

    #[test]
    fn compress_empty_string() {
        let compressed = compress_fast("").expect("compress_fast failed on empty");
        let decompressed = decompress(&compressed).expect("decompress failed on empty");
        assert_eq!(decompressed, "");
    }

    #[test]
    fn compress_unicode_content() {
        let unicode = "日本語テスト 🦀 Ünïcödé — résumé";
        let compressed = compress_fast(unicode).expect("compress_fast failed on unicode");
        let decompressed = decompress(&compressed).expect("decompress failed on unicode");
        assert_eq!(decompressed, unicode);
    }

    #[test]
    fn compress_large_repetitive_content() {
        let large = "a".repeat(100_000);
        let compressed_fast = compress_fast(&large).expect("compress_fast failed");
        let compressed_best = compress_best(&large).expect("compress_best failed");

        // best compression should produce a smaller (or equal) output on compressible data
        assert!(compressed_best.len() <= compressed_fast.len());

        // both must round-trip correctly
        assert_eq!(decompress(&compressed_fast).unwrap(), large);
        assert_eq!(decompress(&compressed_best).unwrap(), large);
    }

    #[test]
    fn compress_output_is_smaller_than_input_for_repetitive_data() {
        let repetitive = "hello world ".repeat(500);
        let compressed = compress_fast(&repetitive).expect("compress_fast failed");
        assert!(
            compressed.len() < repetitive.len(),
            "compressed ({} bytes) should be smaller than original ({} bytes)",
            compressed.len(),
            repetitive.len()
        );
    }

    #[test]
    fn decompress_invalid_data_returns_error() {
        let garbage = b"this is not valid zstd compressed data";
        assert!(decompress(garbage).is_err());
    }
}

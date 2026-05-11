use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

pub fn get_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards (???)")
        .as_secs() as i64
}

pub fn rng_i64(start: i64, end: i64) -> i64 {
    start + ((rng() * (end - start) as f64) as i64)
}

pub fn rng() -> f64 {
    rand::random::<f64>()
}

pub fn construct_digest(data: impl AsRef<[u8]>) -> String {
    // SAFE: returned hash will always be able to be transformed into a [u8; 32]
    let buf: [u8; 32] = Sha256::digest(data).as_slice().try_into().unwrap();

    buf.iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

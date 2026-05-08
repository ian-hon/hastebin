use std::time::{SystemTime, UNIX_EPOCH};

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

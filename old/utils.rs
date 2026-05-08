use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

pub fn generate_name(length: usize) -> String {
    let mut result = "".to_string();
    for _ in 0..length {        
        result += format!("{}{}",
            // async_rng_item(&"aeiouy".chars().collect::<Vec<char>>()),
            // async_rng_item(&"bcdfghjklmnpqrstvwxz".chars().collect::<Vec<char>>())
            async_rng_item(&"ae".chars().collect::<Vec<char>>()),
            async_rng_item(&"bcdf".chars().collect::<Vec<char>>())
        ).as_str();
    }
    result
}

pub fn get_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards (???)")
        .as_secs() as i64
}

pub fn parse_response<T: serde::Serialize>(data: Result<T, T>) -> String {
    match data {
        Ok(d) => format!(r#"{{"type":"success","data":"{}"}}"#, urlencoding::encode(serde_json::to_string(&d).unwrap().as_str()).to_string()),
        Err(e) => format!(r#"{{"type":"fail","error":"{}"}}"#, urlencoding::encode(serde_json::to_string(&e).unwrap().as_str()).to_string())
    }
}

pub fn decode_uri(s: String) -> String {
    urlencoding::decode(&s).unwrap().to_string()
}

// #region rng
pub fn async_rng_range(start: f64, end: f64) -> f64 {
    start + (rand::random::<f64>() * (end - start))
}

pub fn async_rng_range_int(start: i32, end: i32) -> i32 {
    // start + (rand::random::<f64>() * (end - start))
    start + async_rng_int(end - start)
}

pub fn async_rng_range_int_big(start: i64, end: i64) -> i64 {
    start + async_rng_int_big(end - start)
}

pub fn async_rng_bool(i: f64) -> bool {
    rand::random::<f64>() > i
}

pub fn async_rng_float(end: impl Into<f64>) -> f64 {
    rand::random::<f64>() * end.into()
}

pub fn async_rng_int(end: impl Into<i32>) -> i32 {
    (rand::random::<f64>() * (end.into() + 1) as f64) as i32
}

pub fn async_rng_int_big(end: impl Into<i64>) -> i64 {
    (rand::random::<f64>() * (end.into() + 1) as f64) as i64
}

pub fn async_rng_index<T>(inv: &Vec<T>) -> usize {
    async_rng_int(inv.len() as i32 - 1) as usize
}

pub fn async_rng_item<T>(inv: &Vec<T>) -> &T {
    &inv[async_rng_index(inv)]
}
// #endregion

#[derive(FromRow, Debug)]
pub struct Value(pub f64);
// f64 doesnt implement FromRow for some reason???

#[derive(FromRow, Debug, Deserialize)]
pub struct ValueInt(pub i64);

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct ValueString(pub String);
// cant believe i have to do this

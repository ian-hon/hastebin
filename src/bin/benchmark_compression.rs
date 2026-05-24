use std::time::{SystemTime, UNIX_EPOCH};

pub fn compress(data: &str, level: i32) -> Result<Vec<u8>, std::io::Error> {
    zstd::encode_all(data.as_bytes(), level)
}

pub fn decompress(compressed: &[u8]) -> Result<String, std::io::Error> {
    let decompressed = zstd::decode_all(compressed)?;
    String::from_utf8(decompressed)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn get_time() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards (???)")
        .as_micros() as u128
}

fn main() {
    let raw = r#"use sqlx::{Pool, Postgres};

use crate::{
    models::{Comment, Iota},
    utils,
};

impl Comment {
    #[allow(unused)]
    fn new(
        paste_id: i64,
        id: i64,
        content: String,
        author: Option<String>,
        created_at: i64,
        page_index: i64,
        from_row: i64,
        from_column: i64,
        to_row: i64,
        to_column: i64,
    ) -> Self {
        Self {
            paste_id,
            id,
            content,
            author,
            created_at,
            page_index,
            from_row,
            from_column,
            to_row,
            to_column,
        }
    }

    // TODO: encrypt this as well
    pub async fn fetch(id: i64, pool: &Pool<Postgres>) -> Option<Comment> {
        let r = sqlx::query_as::<_, Comment>("SELECT * FROM comment WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await;

        r.ok().flatten()
    }

    pub async fn fetch_from_paste(paste_id: i64, pool: &Pool<Postgres>) -> Vec<Comment> {
        let r = sqlx::query_as::<_, Comment>("SELECT * FROM comment WHERE paste_id = $1")
            .bind(paste_id)
            .fetch_all(pool)
            .await;

        r.unwrap_or_default()
    }

    pub async fn create(
        paste_id: i64,
        content: String,
        author: Option<String>,
        page_index: i64,
        from_row: i64,
        from_column: i64,
        to_row: i64,
        to_column: i64,
        pool: &Pool<Postgres>,
    ) -> Option<i64> {
        if !crate::models::Paste::fetch_internal(paste_id, pool)
            .await?
            .comments_enabled
        {
            return None;
        }

        let id = Self::generate_id(pool).await;

        let _ = sqlx::query("INSERT INTO comment(paste_id, id, content, author, created_at, page_index, from_row, from_column, to_row, to_column) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind(paste_id)
            .bind(id)
            .bind(content)
            .bind(author)
            .bind(utils::get_time()) // see paste.rs on why not postgres' now()
            .bind(page_index)
            .bind(from_row)
            .bind(from_column)
            .bind(to_row)
            .bind(to_column)
            .execute(pool).await;

        Some(id)
    }
}

impl Iota<i64> for Comment {
    async fn fetch_all_ids(pool: &Pool<Postgres>) -> Vec<i64> {
        let r = sqlx::query_scalar::<_, i64>("SELECT id FROM comment")
            .fetch_all(pool)
            .await;

        r.unwrap_or_default()
    }

    async fn generate_id(pool: &Pool<Postgres>) -> i64 {
        // let ids = Self::fetch_all_ids(&pool).await;
        // // O(n), expensive?
        // *ids.iter().max().unwrap_or(&0)
        let candidate = sqlx::query_scalar::<_, i64>("SELECT MAX(id) FROM comment")
            .fetch_optional(pool)
            .await;

        candidate.ok().flatten().unwrap_or(0) + 1
    }
}"#;
    let original_length = raw.len();
    for i in 3..=22 {
        let start = get_time();
        let result = compress(raw, i).unwrap();
        let end = get_time();

        let ratio = (original_length - result.len()) as f64 / original_length as f64;

        println!(
            "({i})\t{}%\t{}x\t\t{}μs\t{} μsl^-1",
            (ratio * 10000f64).round() / 100f64,
            ((original_length as f64 / result.len() as f64) * 10000f64).round() / 10000f64,
            end - start,
            (((end - start) as f64 / result.len() as f64) * 10000f64).round() / 10000f64
        );
    }
}

/*
macbook air m1
(3)     67.28%	3.0561x		748μs	0.7767 μsl^-1
(4)     67.28%	3.0561x		614μs	0.6376 μsl^-1
(5)     68.43%	3.1679x		1155μs	1.2433 μsl^-1
(6)     68.98%	3.2234x		912μs	0.9989 μsl^-1
(7)     69.11%	3.2376x		1578μs	1.736 μsl^-1
(8)     68.98%	3.2234x		793μs	0.8686 μsl^-1
(9)     68.98%	3.2234x		2689μs	2.9452 μsl^-1
(10)	69.01%	3.227x		6028μs	6.6096 μsl^-1
(11)	69.05%	3.2305x		1100μs	1.2075 μsl^-1
(12)	69.05%	3.2305x		9827μs	10.787 μsl^-1
(13)	69.05%	3.2305x		2986μs	3.2777 μsl^-1
(14)	69.05%	3.2305x		8688μs	9.5368 μsl^-1
(15)	69.05%	3.2305x		16203μs	17.7859 μsl^-1
(16)	69.38%	3.2664x		1819μs	2.0189 μsl^-1
(17)	69.66%	3.2956x		4780μs	5.3527 μsl^-1
(18)	69.86%	3.3179x		2765μs	3.1172 μsl^-1
(19)	70.1%	3.3443x		13927μs	15.8261 μsl^-1
(20)	70.1%	3.3443x		38928μs	44.2364 μsl^-1
(21)	70.1%	3.3443x		58273μs	66.2193 μsl^-1
(22)	70.1%	3.3443x		121068μs    137.5773 μsl^-1
*/

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::utils::rng_i64;

// used for id generation
// we want to prioritise the smallest ids
//
// ID_FLOOR is the smallest boundary
// if the randomly generated id cannot fit inside
// that boundary, increase the ceiling by ID_STEPS until ID_CEILING
pub const ID_FLOOR: u32 = 2; // 4: ffff
pub const ID_STEPS: u32 = 2; // 2: ff;  ID_FLOOR + ID_STEPS = ffffff
pub const ID_CEILING: u32 = 16; // i64's max

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Paste {
    pub id: i64,
    pub content: String,

    pub title: Option<String>,
    pub author: Option<String>,

    pub views: i64,

    // in epoch unix (ms)
    pub created_at: i64,
    pub expires_at: Option<i64>,

    // used for diffing and forking (duh)
    pub forked_from: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Comment {
    pub paste_id: i64,

    pub id: i64,
    pub content: String,

    // should we put email here too?
    pub author: Option<String>,

    pub created_at: i64,

    // row, column (y, x)
    // treat as unsigned ints
    pub from_row: i64,
    pub from_column: i64,

    pub to_row: i64,
    pub to_column: i64,
}

// similar to go's enum id system
// https://go.dev/wiki/Iota
pub trait Iota<I: Into<i64> + Copy> {
    // https://blog.rust-lang.org/2023/12/28/Rust-1.75.0/#async-fn-and-return-position-impl-trait-in-traits
    // https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/#where-the-gaps-lie
    #![allow(async_fn_in_trait)]
    async fn fetch_all_ids(pool: &Pool<Postgres>) -> Vec<I>;
    async fn generate_id(pool: &Pool<Postgres>) -> i64 {
        let mut highest = i64::MIN;

        let ids: HashSet<i64> = Self::fetch_all_ids(pool)
            .await
            .iter()
            .map(|&x| {
                let x: i64 = x.into();
                highest = highest.max(x);

                return x;
            })
            .collect();

        // start with 0-ff, (0-f^2)
        // 0-ffff, (0-f^4)
        // 0-ffffff, (0-f^6)
        // 0-ffffffff, (0-f^8)
        // until we hit the ceiling
        for level in 0..ID_CEILING {
            let floor = 16_i64.pow(level * ID_STEPS);
            let ceiling = 16_i64.pow((level + 1) * ID_STEPS);
            for _ in floor..ceiling {
                let candidate = rng_i64(floor, ceiling);

                if !ids.contains(&candidate) {
                    return candidate;
                }
            }
        }

        // if this panics, hastebin is either really
        // popular or im bad at this
        highest + 1
    }
}

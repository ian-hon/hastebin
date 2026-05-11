use std::collections::HashSet;

use sqlx::{Pool, Postgres};

use crate::{
    models::{Iota, Paste},
    utils::{self, rng_i64},
};

// used for id generation
// we want to prioritise the smallest ids
//
// ID_FLOOR is the smallest boundary
// if the randomly generated id cannot fit inside
// that boundary, increase the ceiling by ID_STEPS until ID_CEILING
pub const PASTE_ID_FLOOR: u32 = 2; // 4: ffff
pub const PASTE_ID_STEPS: u32 = 2; // 2: ff;  ID_FLOOR + ID_STEPS = ffffff
pub const PASTE_ID_CEILING: u32 = 16; // i64's max

impl Paste {
    #[allow(unused)]
    fn new(
        id: i64,
        content: String,
        title: Option<String>,
        author: Option<String>,
        checksum_passphrase: Option<String>,
        views: i64,
        comments_enabled: bool,
        created_at: i64,
        expires_at: Option<i64>,
        forked_from: Option<i64>,
    ) -> Self {
        Self {
            id,
            content,
            title,
            author,
            checksum_passphrase,
            views,
            comments_enabled,
            created_at,
            expires_at,
            forked_from,
        }
    }

    pub async fn fetch(id: i64, pool: &Pool<Postgres>) -> Option<Paste> {
        let _ = sqlx::query("UPDATE paste SET views = views + 1 WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await;

        // https://docs.rs/sqlx/latest/sqlx/fn.query_as.html#example-map-rows-using-tuples
        // paste.checksum_passphrase,
        // paste.views, paste.comments_enabled, paste.created_at, paste.expires_at, paste.forked_from
        let r = sqlx::query_as::<_, Paste>("SELECT * FROM paste WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await;

        r.ok().flatten()
    }

    pub async fn create(
        content: String,
        title: Option<String>,
        author: Option<String>,
        checksum_passphrase: Option<String>,
        comments_enabled: bool,
        mut expires_at: Option<i64>,
        forked_from: Option<i64>,
        pool: &Pool<Postgres>,

        soft_limit: usize,
        default_expiry_days: u32,
    ) -> Option<i64> {
        let id = Self::generate_id(pool).await;

        // if the content size is too big, we impose the minimum expiry
        if content.len() >= soft_limit {
            expires_at = Some(
                (utils::get_time() + (86400 * (default_expiry_days as i64)))
                    .min(expires_at.unwrap_or(i64::MAX)),
            );
        }

        if let Ok(_) = sqlx::query("INSERT INTO paste(id, content, title, author, checksum_passphrase, views, comments_enabled, created_at, expires_at, forked_from) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind(id)
            .bind(content)
            .bind(title)
            .bind(author)
            .bind(checksum_passphrase.map(|c| utils::construct_digest(c)))
            .bind(0)
            .bind(comments_enabled)
            // using Postgres' now() is good practice; but why not here?
            //
            // there were one or two cases in my past experience where Supabase's server
            // returned times in different timezones; possibly due to regional nodes + maintenance periods?
            //
            // seems like a breaking issue on their side but ill play it safe and compute
            // time locally, since im running only a singular centralised instance
            .bind(utils::get_time())
            .bind(expires_at)
            .bind(forked_from)
            .execute(pool).await {
                Some(id)
            } else {
                None
            }
    }
}
impl Iota<i64> for Paste {
    async fn fetch_all_ids(pool: &Pool<Postgres>) -> Vec<i64> {
        // https://docs.rs/sqlx/latest/sqlx/fn.query_scalar.html
        // https://docs.rs/sqlx/latest/sqlx/query/struct.QueryAs.html#method.fetch_all
        let r = sqlx::query_scalar::<_, i64>("SELECT id FROM paste")
            .fetch_all(pool)
            .await;

        r.unwrap_or_default()
    }

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
        for level in 0..PASTE_ID_CEILING {
            let floor = 16_i64.pow(level * PASTE_ID_STEPS);
            let ceiling = 16_i64.pow((level + 1) * PASTE_ID_STEPS);
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

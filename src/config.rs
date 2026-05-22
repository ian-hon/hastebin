use std::env;

use api::Config;
use dotenv::dotenv;

pub fn from_env() -> anyhow::Result<Config> {
    dotenv().ok();

    Ok(Config {
        database_url: env::var("DATABASE_URL")?,
        port: env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()?,
        size_soft_limit: env::var("SIZE_SOFT_LIMIT")
            .unwrap_or_else(|_| "20000".to_string())
            .parse::<usize>()?,
        default_expiry_days: env::var("DEFAULT_EXPIRY_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse()?,
        cleanup_interval: env::var("CLEANUP_INTERVAL")
            .unwrap_or_else(|_| "86400".to_string())
            .parse()?,
    })
}

use std::env;

use dotenv::dotenv;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub port: u16,

    // the expiry applies only to pastes over the limit
    pub size_soft_limit: usize,
    pub default_expiry_days: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        println!("huh");
        dotenv().ok();
        println!("huh");

        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse::<u16>()?,
            size_soft_limit: env::var("SIZE_SOFT_LIMIT")
                .unwrap_or_else(|_| "52488".to_string())
                .parse::<usize>()?,
            default_expiry_days: env::var("DEFAULT_EXPIRY_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()?,
        })
    }
}

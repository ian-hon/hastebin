use anyhow::{Context, Result};
use csv::ReaderBuilder;
use dotenv::dotenv;
use engine::compression;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs::File;

const SIZE_SOFT_LIMIT: usize = 20_000;
const DEFAULT_EXPIRY_DAYS: u32 = 7;
const CSV_PATH: &str = "archive/raw.csv";

#[derive(Debug, Serialize, Deserialize)]
struct PasteFile {
    #[serde(rename = "fileName")]
    file_name: String,
    content: String,
}

#[derive(Debug)]
struct OldPaste {
    id: i64,
    content: String,
    signature: String,
    views: i64,
    timestamp: i64,
}

#[derive(Debug)]
struct MigrationStats {
    total: usize,
    success: usize,
    ignored: usize,
    parse_failed: usize,
    compression_failed: usize,
    verification_failed: usize,
    insert_failed: usize,
}

impl MigrationStats {
    fn new() -> Self {
        Self {
            total: 0,
            success: 0,
            ignored: 0,
            parse_failed: 0,
            compression_failed: 0,
            verification_failed: 0,
            insert_failed: 0,
        }
    }

    fn print(&self) {
        println!("\n==== Migration Statistics ====");
        println!("Total processed:       {}", self.total);
        println!("✓ Success:             {}", self.success);
        println!("⊘ Ignored (parse):     {}", self.ignored);
        println!("✗ Parse failed:        {}", self.parse_failed);
        println!("✗ Compression failed:  {}", self.compression_failed);
        println!("✗ Verification failed: {}", self.verification_failed);
        println!("✗ Insert failed:       {}", self.insert_failed);
        println!("==============================");
    }
}

fn parse_old_content(content: &str) -> Result<Vec<PasteFile>> {
    // Old format: [["fileName","content"],["fileName2","content2"]]
    let parsed: Vec<Vec<String>> =
        serde_json::from_str(content).context("Failed to parse old content as 2D array")?;

    if parsed.is_empty() {
        anyhow::bail!("Empty content array");
    }

    let mut files = Vec::new();
    for entry in parsed {
        if entry.len() != 2 {
            anyhow::bail!("Invalid entry format, expected [fileName, content]");
        }
        files.push(PasteFile {
            file_name: entry[0].clone(),
            content: entry[1].clone(),
        });
    }

    Ok(files)
}

fn parse_csv_row(record: &csv::StringRecord) -> Result<OldPaste> {
    if record.len() < 5 {
        anyhow::bail!("Invalid CSV row: expected 5 columns, got {}", record.len());
    }

    Ok(OldPaste {
        id: record[0].parse().context("Failed to parse ID")?,
        content: record[1].to_string(),
        signature: record[2].to_string(),
        views: record[3].parse().context("Failed to parse views")?,
        timestamp: record[4].parse().context("Failed to parse timestamp")?,
    })
}

async fn migrate_paste(
    old: OldPaste,
    pool: &sqlx::PgPool,
    stats: &mut MigrationStats,
) -> Result<()> {
    // Parse old content to new format
    let files = match parse_old_content(&old.content) {
        Ok(f) => f,
        Err(e) => {
            println!("  [IGNORED] ID {}: {}", old.id, e);
            stats.ignored += 1;
            return Ok(());
        }
    };

    // Convert to new JSON format
    let new_content_json =
        serde_json::to_string(&files).context("Failed to serialize new content")?;

    // Compress
    let compressed = match compression::compress_fast(&new_content_json) {
        Ok(c) => c,
        Err(e) => {
            println!("  [COMPRESSION FAILED] ID {}: {}", old.id, e);
            stats.compression_failed += 1;
            return Ok(());
        }
    };

    // Verify by decompressing
    let decompressed = match compression::decompress(&compressed) {
        Ok(d) => d,
        Err(e) => {
            println!("  [VERIFICATION FAILED] ID {}: {}", old.id, e);
            stats.verification_failed += 1;
            return Ok(());
        }
    };

    // Verify content matches
    if decompressed != new_content_json {
        println!(
            "  [VERIFICATION FAILED] ID {}: decompressed content doesn't match",
            old.id
        );
        stats.verification_failed += 1;
        return Ok(());
    }

    // Calculate expiry
    let expires_at = if new_content_json.len() >= SIZE_SOFT_LIMIT {
        Some(old.timestamp + (86400 * DEFAULT_EXPIRY_DAYS as i64))
    } else {
        None
    };

    // Insert into database
    let author = if old.signature.is_empty() {
        None
    } else {
        Some(old.signature.clone())
    };

    let compressed_len = compressed.len();

    let result = sqlx::query(
        "INSERT INTO paste (id, content, title, author, checksum_passphrase, views, comments_enabled, created_at, expires_at, forked_from) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(old.id)
    .bind(compressed)
    .bind(None::<String>) // title
    .bind(author)
    .bind(None::<String>) // checksum_passphrase
    .bind(old.views)
    .bind(false) // comments_enabled
    .bind(old.timestamp)
    .bind(expires_at)
    .bind(None::<i64>) // forked_from
    .execute(pool)
    .await;

    match result {
        Ok(_) => {
            println!(
                "  [SUCCESS] ID {}: {} files, {} bytes compressed",
                old.id,
                files.len(),
                compressed_len
            );
            stats.success += 1;
        }
        Err(e) => {
            println!("  [INSERT FAILED] ID {}: {}", old.id, e);
            stats.insert_failed += 1;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut limit = 5;

    for i in 0..args.len() {
        if args[i] == "--limit" && i + 1 < args.len() {
            limit = args[i + 1].parse().context("Invalid limit value")?;
        }
    }

    println!("=== Hastebin Migration Tool ===");
    println!("CSV file: {}", CSV_PATH);
    println!("Limit: {} rows", limit);
    println!();

    // Connect to database
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL not found in environment")?;

    println!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    println!("Connected successfully!");
    println!();

    // Open and read CSV
    let file = File::open(CSV_PATH).context(format!("Failed to open {}", CSV_PATH))?;

    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);

    let mut stats = MigrationStats::new();

    println!("Starting migration...");
    println!();

    // Process each row
    for (idx, result) in reader.records().enumerate() {
        if idx >= limit {
            println!("Reached limit of {} rows", limit);
            break;
        }

        stats.total += 1;

        let record = match result {
            Ok(r) => r,
            Err(e) => {
                println!("  [PARSE FAILED] Row {}: {}", idx + 1, e);
                stats.parse_failed += 1;
                continue;
            }
        };

        let old_paste = match parse_csv_row(&record) {
            Ok(p) => p,
            Err(e) => {
                println!("  [PARSE FAILED] Row {}: {}", idx + 1, e);
                stats.parse_failed += 1;
                continue;
            }
        };

        if let Err(e) = migrate_paste(old_paste, &pool, &mut stats).await {
            println!("  [ERROR] Row {}: {}", idx + 1, e);
        }
    }

    stats.print();

    Ok(())
}

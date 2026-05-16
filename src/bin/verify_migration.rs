use csv::Reader;
use engine::compression;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
struct ResultRecord {
    id: i64,
    content: String,
    title: String,
    author: String,
    checksum_passphrase: String,
    views: i32,
    comments_enabled: bool,
    created_at: i64,
    expires_at: String,
    forked_from: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BufferData {
    #[serde(rename = "type")]
    buffer_type: String,
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PasteFile {
    #[serde(rename = "fileName")]
    file_name: String,
    content: String,
}

pub fn main() {
    println!("Verifying migration results...\n");

    let file = File::open("./archive/result.csv").expect("Failed to open result.csv");
    let mut reader = Reader::from_reader(file);

    let mut verified = 0;
    let mut errors = 0;

    for (idx, result) in reader.deserialize().enumerate() {
        let record: ResultRecord = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Failed to parse record {}: {}", idx + 1, e);
                errors += 1;
                continue;
            }
        };

        // Parse the buffer JSON
        let buffer: BufferData = match serde_json::from_str(&record.content) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Failed to parse buffer JSON for ID {}: {}", record.id, e);
                errors += 1;
                continue;
            }
        };

        // Verify buffer type
        if buffer.buffer_type != "Buffer" {
            eprintln!(
                "Invalid buffer type for ID {}: {}",
                record.id, buffer.buffer_type
            );
            errors += 1;
            continue;
        }

        // Decompress
        let decompressed = match compression::decompress(&buffer.data) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to decompress for ID {}: {}", record.id, e);
                errors += 1;
                continue;
            }
        };

        // Parse as PasteFile array
        let files: Vec<PasteFile> = match serde_json::from_str(&decompressed) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Failed to parse decompressed content for ID {}: {}",
                    record.id, e
                );
                errors += 1;
                continue;
            }
        };

        if files.is_empty() {
            eprintln!("Empty files array for ID {}", record.id);
            errors += 1;
            continue;
        }

        verified += 1;

        // Show first 3 records as examples
        if verified <= 3 {
            println!("✓ Record ID {}: {} file(s)", record.id, files.len());
            for file in &files {
                let preview = if file.content.len() > 50 {
                    format!("{}...", &file.content[..50])
                } else {
                    file.content.clone()
                };
                println!("  - {}: {}", file.file_name, preview);
            }
            println!();
        }

        if verified % 1000 == 0 {
            println!("Verified {} records...", verified);
        }
    }

    println!("\n=== Verification Complete ===");
    println!("Successfully verified: {}", verified);
    println!("Errors:               {}", errors);
    println!("============================");
}

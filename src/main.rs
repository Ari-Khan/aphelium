use polars::prelude::*;
use std::fs;
use std::path::Path;
use yahoo_finance_api as yahoo;
use chrono::{DateTime, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create necessary folders if they don't exist
    fs::create_dir_all("data")?;
    fs::create_dir_all("results")?;

    // The list of tickers to harvest
    let tickers = vec!["AAPL", "MSFT", "GOOGL", "AMZN", "TSLA", "SPY", "QQQ", "DIA"];

    println!("🚀 Aphelium Harvester Initialized...");

    for ticker in tickers {
        let file_path = format!("data/{}.parquet", ticker);

        // PROGRESS CHECK: Skip if we already have this data
        if Path::new(&file_path).exists() {
            println!("⏩ Skipping {}: Data already in vault.", ticker);
            continue;
        }

        println!("📥 Fetching data for {}...", ticker);
        
        match download_to_parquet(ticker, &file_path).await {
            Ok(_) => println!("✅ Successfully saved {}.", ticker),
            Err(e) => eprintln!("❌ Error downloading {}: {}", ticker, e),
        }
    }

    println!("\n⭐ Harvest complete. All tickers are stored in /data.");
    Ok(())
}

async fn download_to_parquet(ticker: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let provider = yahoo::YahooConnector::new()?;
    
    // Fetch max history (usually goes back to IPO or 1980s)
    let response = provider.get_quote_range(ticker, "1d", "max").await?;
    let quotes = response.quotes()?;

    // Map Yahoo data to vectors for Polars
    let dates: Vec<i64> = quotes.iter().map(|q| q.timestamp as i64).collect();
    let opens: Vec<f64> = quotes.iter().map(|q| q.open).collect();
    let highs: Vec<f64> = quotes.iter().map(|q| q.high).collect();
    let lows: Vec<f64> = quotes.iter().map(|q| q.low).collect();
    let closes: Vec<f64> = quotes.iter().map(|q| q.close).collect();
    let volumes: Vec<f64> = quotes.iter().map(|q| q.volume as f64).collect();

    // Create the DataFrame
    let mut df = df!(
        "timestamp" => dates,
        "open" => opens,
        "high" => highs,
        "low" => lows,
        "close" => closes,
        "volume" => volumes,
    )?;

    // Save to the Vault (Parquet)
    let mut file = fs::File::create(path)?;
    ParquetWriter::new(&mut file).finish(&mut df)?;

    Ok(())
}
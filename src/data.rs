use polars::prelude::*;
use std::fs;
use std::path::Path;
use yahoo_finance_api as yahoo;

pub async fn harvest_max_history(ticker: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(path).exists() { return Ok(()); }

    let provider = yahoo::YahooConnector::new()?;
    let resp = provider.get_quote_range(ticker, "1d", "max").await?;
    let quotes = resp.quotes()?;

    let mut df = df!(
        "timestamp" => quotes.iter().map(|x| x.timestamp as i64).collect::<Vec<i64>>(),
        "close" => quotes.iter().map(|x| x.close).collect::<Vec<f64>>()
    )?;

    fs::create_dir_all("data")?;
    let mut file = fs::File::create(path)?;
    ParquetWriter::new(&mut file).finish(&mut df)?;
    
    println!("Harvested {} years of history ({} days).", df.height() / 252, df.height());
    Ok(())
}
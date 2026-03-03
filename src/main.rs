mod data;
mod generator;
mod executor;

use polars::prelude::*;
use std::fs::File;
use tokio;

struct StrategyRank {
    params: generator::StrategyParams,
    profit: f64,
    sharpe: f64,
    max_dd: f64,
    trades: usize,
    cagr: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ticker = "SPY";
    let path = "data/SPY.parquet";

    data::harvest_max_history(ticker, path).await?;
    let file = File::open(path)?;
    let df = ParquetReader::new(file).finish()?;
    
    let space = generator::create_search_space(); 
    eprintln!("Testing {} Portfolio Variations...", space.len());

    let mut leaderboard: Vec<StrategyRank> = Vec::new();

    for p in space {
        if let Some((ret, sharpe, mdd, count, cagr)) = executor::evaluate(&df, &p) {
            leaderboard.push(StrategyRank { 
                params: p, 
                profit: ret, 
                sharpe, 
                max_dd: mdd, 
                trades: count, 
                cagr 
            });
        }
    }

    leaderboard.sort_by(|a, b| b.cagr.partial_cmp(&a.cagr).unwrap_or(std::cmp::Ordering::Equal));

    println!("{:<24} | {:<5} | {:<4} | {:<8} | {:<8} | {:<8} | {:<8} | {:<8} | {:<6}", 
             "STRATEGY (Buy -> Sell)", "SLOTS", "EXIT", "TOTAL %", "ANNUAL %", "SHARPE", "CALMAR", "MAX DD", "TRADES");
    println!("{}", "-".repeat(125));

    for s in leaderboard.iter().take(5000) {
        let exit_str = if s.params.exit_days == 0 { 
            "SIG".to_string() 
        } else { 
            format!("{}d", s.params.exit_days) 
        };
        
        let calmar = if s.max_dd.abs() > 0.0001 { 
            s.cagr / s.max_dd.abs() 
        } else { 
            0.0 
        };

        println!("B:{:>2}/{:>4.1}% -> S:{:>2}/{:>4.1}% | {:>5} | {:>4} | {:>7.1}% | {:>7.1}% | {:>8.2} | {:>8.2} | {:>7.1}% | {:>6}",
            s.params.buy_lb, s.params.buy_thr * 100.0, 
            s.params.sell_lb, s.params.sell_thr * 100.0,
            s.params.max_slots,
            exit_str,
            s.profit * 100.0, 
            s.cagr * 100.0,
            s.sharpe, 
            calmar, 
            s.max_dd * 100.0, 
            s.trades);
    }

    Ok(())
}
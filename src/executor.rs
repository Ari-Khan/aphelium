use polars::prelude::*;

struct ActiveTrade {
    entry_price: f64,
    entry_idx: usize,
}

pub fn evaluate(df: &DataFrame, params: &super::generator::StrategyParams) -> Option<(f64, f64, f64, usize, f64)> {
    let prices = df.column("close").ok()?.f64().ok()?;
    let timestamps = df.column("timestamp").ok()?.i64().ok()?;
    let len = prices.len();

    let mut trade_returns = Vec::new();
    let mut active_trades: Vec<ActiveTrade> = Vec::new();
    let mut first_ts = 0;
    let mut last_ts = 0;

    let mut current_balance = 1.0;
    let mut peak = 1.0;
    let mut max_drawdown = 0.0;

    for i in 63..len {
        let current_price = prices.get(i)?;

        let mut j = 0;
        while j < active_trades.len() {
            let trade = &active_trades[j];
            let lb_price = prices.get(i - params.sell_lb as usize)?;
            let diff = (current_price / lb_price) - 1.0;
            
            let signal_exit = if params.sell_thr < 0.0 { diff < params.sell_thr } else { diff > params.sell_thr };
            let time_exit = params.exit_days > 0 && (i - trade.entry_idx) >= params.exit_days as usize;

            if signal_exit || time_exit {
                let pnl = (current_price / trade.entry_price) - 1.0;
                
                current_balance *= 1.0 + (pnl / params.max_slots as f64);
                
                if current_balance > peak { peak = current_balance; }
                let dd = (current_balance - peak) / peak;
                if dd < max_drawdown { max_drawdown = dd; }

                trade_returns.push(pnl);
                active_trades.remove(j);
                last_ts = timestamps.get(i)?;
            } else {
                j += 1;
            }
        }

        if active_trades.len() < params.max_slots {
            let lb_price = prices.get(i - params.buy_lb as usize)?;
            let diff = (current_price / lb_price) - 1.0;
            let triggered = if params.buy_thr < 0.0 { diff < params.buy_thr } else { diff > params.buy_thr };

            if triggered {
                active_trades.push(ActiveTrade { entry_price: current_price, entry_idx: i });
                if first_ts == 0 { first_ts = timestamps.get(i)?; }
            }
        }
    }

    let count = trade_returns.len();
    if count < 15 { return None; }

    let total_ret = current_balance - 1.0;
    let years = (last_ts - first_ts) as f64 / 31_557_600.0;
    let cagr = if years > 0.1 && current_balance > 0.0 { f64::powf(current_balance, 1.0 / years) - 1.0 } else { 0.0 };
    
    let mean = trade_returns.iter().sum::<f64>() / count as f64;
    let std = (trade_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / count as f64).sqrt().max(0.0001);
    let sharpe = (mean / std) * 15.87;

    Some((total_ret, sharpe, max_drawdown, count, cagr))
}
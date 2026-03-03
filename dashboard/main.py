import pandas as pd
import plotly.graph_objects as go
import re
import os

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PARQUET_PATH = os.path.join(BASE_DIR, "data", "SPY.parquet")
RESULTS_PATH = os.path.join(BASE_DIR, "results", "results.txt")

def translate_to_english(label, exit_days):
    m = re.search(r"B:(\d+)/(.+?)% -> S:(\d+)/(.+?)%", label)
    if m:
        b_days, b_th, s_days, s_th = m.groups()
        b_type = "Dip" if float(b_th) < 0 else "Pump"
        s_type = "Drop" if float(s_th) < 0 else "Gain"
        exit_str = f" or {exit_days}d Max" if exit_days > 0 else ""
        return f"Buy {abs(float(b_th))}% {b_type} ({b_days}d) | Exit {abs(float(s_th))}% {s_type}{exit_str}"
    return label

def run_simulation(df, b_lb, b_th, s_lb, s_th, exit_days, slots=1):
    balance = 1000.0
    equity_curve = [balance]
    active_trades = []
    prices = df['close'].values
    
    for i in range(63, len(prices)):
        current_price = prices[i]
        new_active = []
        for entry_price, entry_idx in active_trades:
            lb_price = prices[i - s_lb]
            diff = (current_price / lb_price) - 1.0
            signal_exit = diff < s_th if s_th < 0 else diff > s_th
            time_exit = exit_days > 0 and (i - entry_idx) >= exit_days
            if signal_exit or time_exit:
                balance *= (1.0 + (((current_price / entry_price) - 1.0) / slots))
            else:
                new_active.append((entry_price, entry_idx))
        active_trades = new_active
        if len(active_trades) < slots:
            lb_price = prices[i - b_lb]
            diff = (current_price / lb_price) - 1.0
            if (diff < b_th if b_th < 0 else diff > b_th):
                active_trades.append((current_price, i))
        equity_curve.append(balance)
    return equity_curve

if __name__ == "__main__":
    if not os.path.exists(PARQUET_PATH):
        exit()

    raw_df = pd.read_parquet(PARQUET_PATH)
    raw_df['timestamp'] = pd.to_datetime(raw_df['timestamp'], unit='s')
    
    with open(RESULTS_PATH, 'r') as f:
        lines = [line for line in f.readlines() if "B:" in line]
    
    fig = go.Figure()
    dates = raw_df['timestamp'].iloc[63:]
    initial_cash = 1000.0

    for line in lines[:5]:
        parts = [p.strip() for p in line.split('|')]
        raw_label = parts[0]
        exit_days = int(parts[2].replace('d', '')) if 'd' in parts[2] else 0
        eng_label = translate_to_english(raw_label, exit_days)
        m = re.search(r"B:(\d+)/(.+?)% -> S:(\d+)/(.+?)%", raw_label)
        if m:
            curve = run_simulation(raw_df, int(m.group(1)), float(m.group(2))/100, 
                                   int(m.group(3)), float(m.group(4))/100, 
                                   exit_days, int(parts[1]))
            fig.add_trace(go.Scatter(
                x=dates, y=curve[:len(dates)], 
                name=eng_label,
                hovertemplate="<b>" + eng_label + "</b><br>Value: %{y:$,.2f}<extra></extra>"
            ))

    spy_hold = (raw_df['close'].iloc[63:] / raw_df['close'].iloc[63]) * initial_cash
    fig.add_trace(go.Scatter(
        x=dates, y=spy_hold, name="Standard S&P 500 (Buy & Hold)", 
        line=dict(dash='dash', color='gray'),
        hovertemplate="<b>S&P 500</b><br>Value: %{y:$,.2f}<extra></extra>"
    ))

    fig.update_layout(
        title="<b>Institutional Strategy Performance</b><br>Wealth Growth of $1000",
        template="plotly_dark",
        
        paper_bgcolor="black", 
        plot_bgcolor="black",
        
        xaxis_title="Year",
        yaxis_title="Account Balance ($)",
        hovermode="x unified",
        height=676,
        
        legend=dict(
            orientation="h",
            yanchor="bottom",
            y=-0.6,
            xanchor="center",
            x=0.5,
            font=dict(size=13)
        ),
        
        yaxis=dict(
            rangemode="tozero", 
            tickformat="$,.0f",
            gridcolor="#222",
            zerolinecolor="#444"
        ),
        
        xaxis=dict(gridcolor="#222"),
        
        margin=dict(b=250, t=100, l=10, r=10) 
    )

    fig.show(config={'responsive': True})

    fig.show()
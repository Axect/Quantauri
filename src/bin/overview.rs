use peroxide::fuga::*;
use quantauri::api::download_stocks;
use quantauri::ta::{adx_dmi, cci, divergence, ema, macd, rsi, sma, stochastic, wma};
use std::env::args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let symbol = args().nth(1).unwrap_or("005930.KS".to_string());
    let from = "2022-01-09 00:00:00 +09"; // For cushion
    let to = "2023-10-12 00:00:00 +09";
    let symbol_wrap = vec![symbol.clone()];

    let stock = download_stocks(&symbol_wrap, from, to).await?;
    let stock = stock[0].clone();
    let df = stock.to_dataframe();
    let date: Vec<String> = df["date"].to_vec();
    let close: Vec<f64> = df["close"].to_vec();
    let high: Vec<f64> = df["high"].to_vec();
    let low: Vec<f64> = df["low"].to_vec();
    let tp = close
        .iter()
        .zip(high.iter())
        .zip(low.iter())
        .map(|((c, h), l)| (c + h + l) / 3f64)
        .collect::<Vec<f64>>();
    let (tp_div, tp_slope) = divergence(&tp);
    let sma_ = sma(&tp, 20);
    let ema_ = ema(&tp, 20);
    let wma_ = wma(&tp, 20);
    let rsi_ = rsi(&close, 14);
    let rsi_signal = ema(&rsi_, 9);
    let (rsi_div, rsi_slope) = divergence(&rsi_);
    let macd_ = macd(&close, 12, 26);
    let macd_signal = ema(&macd_, 9);
    let (adx_, di_plus, di_minus) = adx_dmi(&high, &low, &close, 14);
    let (k, d) = stochastic(&high, &low, &close, 14, 3);
    let cci_ = cci(&high, &low, &close, 20);

    // Result
    let date = date[240..].to_vec();
    let tp = tp[240..].to_vec();
    let (tp_div, tp_slope) = (tp_div[240..].to_vec(), tp_slope[240..].to_vec());
    let sma_ = sma_[240..].to_vec();
    let ema_ = ema_[240..].to_vec();
    let wma_ = wma_[240..].to_vec();
    let rsi_ = rsi_[240..].to_vec();
    let rsi_signal = rsi_signal[240..].to_vec();
    let (rsi_div, rsi_slope) = (rsi_div[240..].to_vec(), rsi_slope[240..].to_vec());
    let macd_ = macd_[240..].to_vec();
    let macd_signal = macd_signal[240..].to_vec();
    let adx_ = adx_[240..].to_vec();
    let di_plus = di_plus[240..].to_vec();
    let di_minus = di_minus[240..].to_vec();
    let k = k[240..].to_vec();
    let d = d[240..].to_vec();
    let cci_ = cci_[240..].to_vec();

    let mut dg = DataFrame::new(vec![]);
    dg.push("date", Series::new(date));
    dg.push("tp", Series::new(tp));
    dg.push("tp_div", Series::new(tp_div));
    dg.push("tp_slope", Series::new(tp_slope));
    dg.push("sma", Series::new(sma_));
    dg.push("ema", Series::new(ema_));
    dg.push("wma", Series::new(wma_));
    dg.push("rsi", Series::new(rsi_));
    dg.push("rsi_signal", Series::new(rsi_signal));
    dg.push("rsi_div", Series::new(rsi_div));
    dg.push("rsi_slope", Series::new(rsi_slope));
    dg.push("macd", Series::new(macd_));
    dg.push("macd_signal", Series::new(macd_signal));
    dg.push("adx", Series::new(adx_));
    dg.push("di_plus", Series::new(di_plus));
    dg.push("di_minus", Series::new(di_minus));
    dg.push("k", Series::new(k));
    dg.push("d", Series::new(d));
    dg.push("cci", Series::new(cci_));

    dg.print();

    dg.write_parquet(
        &format!("./data/{}.parquet", symbol),
        CompressionOptions::Uncompressed,
    )?;

    Ok(())
}

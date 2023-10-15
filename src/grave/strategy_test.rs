use fmp::api::{download_stocks, Quote, HistoricalChart};
use fmp::strategy::*;
use fmp::ta::*;
use peroxide::fuga::*;
use std::env::args;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let symbol = args().nth(1).unwrap_or("005930.KS".to_string());
    let symbol_wrapper = vec![symbol.clone()];
    let from = "2018-01-01 00:00:00 +09";
    let to = "2023-10-10 00:00:00 +09";

    let stock = download_stocks(&symbol_wrapper, from, to).await?;
    let stock = stock[0].clone();

    //let tnx_wrapper = vec!["^TNX".to_string()];
    //let tnx = download_stocks(&tnx_wrapper, from, to).await?;
    //let tnx = tnx[0].clone();
    //let tnx = tnx.get_close_vec();
    //let mut risk_free = tnx.fmap(|x| (1f64 + x / 100f64).powf(1f64 / 252f64) - 1f64);
    //risk_free[0..120].fill(0f64);
    let kospi_wrapper = vec!["^KS11".to_string()];
    let kospi = download_stocks(&kospi_wrapper, from, to).await?;
    let kospi = kospi[0].clone();
    let kospi = kospi.get_close_vec();
    let init_kospi = kospi[0];
    let bnh_kospi = BuyAndHold::new(init_kospi, &kospi);
    let risk_free = bnh_kospi.daily_return();

    let mut df = stock.to_dataframe();
    let open: Vec<f64> = df["open"].to_vec();
    let close: Vec<f64> = df["close"].to_vec();
    let high: Vec<f64> = df["high"].to_vec();
    let low: Vec<f64> = df["low"].to_vec();

    df.print();

    // Buy and Hold
    let init = open[0];
    let bnh = BuyAndHold::new(init, &close);
    let bnh_cr = bnh.cumulative_return();
    let bnh_vol = bnh.roll_volatility(120);
    let bnh_sr = bnh.roll_sharpe_ratio(&risk_free, 120);
    let bnh_dd = bnh.drawdown();
    df.push("bnh_cr", Series::new(bnh_cr));
    df.push("bnh_vol", Series::new(bnh_vol));
    df.push("bnh_sr", Series::new(bnh_sr));
    df.push("bnh_dd", Series::new(bnh_dd));

    // MA Crossover
    let ma_co = MACrossover::new(20, 50, &close);
    let ma_co_cr = ma_co.cumulative_return();
    let ma_co_vol = ma_co.roll_volatility(120);
    let ma_co_sr = ma_co.roll_sharpe_ratio(&risk_free, 120);
    let ma_co_dd = ma_co.drawdown();
    df.push("ma_co_cr", Series::new(ma_co_cr));
    df.push("ma_co_vol", Series::new(ma_co_vol));
    df.push("ma_co_sr", Series::new(ma_co_sr));
    df.push("ma_co_dd", Series::new(ma_co_dd));

    // MACD + ADX
    let macd_adx = MACD_ADX::new(0.5, &high, &low, &close);
    let macd_adx_cr = macd_adx.cumulative_return();
    let macd_adx_vol = macd_adx.roll_volatility(120);
    let macd_adx_sr = macd_adx.roll_sharpe_ratio(&risk_free, 120);
    let macd_adx_dd = macd_adx.drawdown();
    df.push("macd_adx_cr", Series::new(macd_adx_cr));
    df.push("macd_adx_vol", Series::new(macd_adx_vol));
    df.push("macd_adx_sr", Series::new(macd_adx_sr));
    df.push("macd_adx_dd", Series::new(macd_adx_dd));

    df.write_parquet(
        &format!("data/{}_strategy.parquet", symbol),
        CompressionOptions::Uncompressed,
    )?;

    let mut dg = DataFrame::new(vec![]);
    dg.push(
        "Strategy",
        Series::new(vec![
            "BnH".to_string(),
            "MA_CO".to_string(),
            "MACD_ADX".to_string(),
        ]),
    );
    dg.push(
        "CAGR",
        Series::new(vec![bnh.cagr(), ma_co.cagr(), macd_adx.cagr()]),
    );
    dg.push(
        "Volatility",
        Series::new(vec![
            bnh.volatility(),
            ma_co.volatility(),
            macd_adx.volatility(),
        ]),
    );
    dg.push(
        "Sharpe",
        Series::new(vec![
            bnh.sharpe_ratio(&risk_free),
            ma_co.sharpe_ratio(&risk_free),
            macd_adx.sharpe_ratio(&risk_free),
        ]),
    );
    dg.push(
        "MDD",
        Series::new(vec![bnh.mdd(), ma_co.mdd(), macd_adx.mdd()]),
    );

    dg.print();

    dg.write_parquet(
        &format!("data/{}_summary.parquet", symbol),
        CompressionOptions::Uncompressed,
    )?;

    Ok(())
}

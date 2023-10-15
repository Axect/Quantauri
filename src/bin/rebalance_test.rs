use peroxide::fuga::*;
use quantauri::{
    strategy::{BuyAndHold, PeriodicRebalance, ThresholdRebalance},
    trade::Backtester,
};
use std::collections::HashMap;
use std::env::args;

const PERIOD: [usize; 6] = [1, 5, 20, 60, 120, 240];
const THRESHOLD: [f64; 6] = [1f64, 5f64, 10f64, 15f64, 20f64, 30f64];

// Tickers
// - 005930.KS  : Samsung Electronics
// - SPY        : S&P 500
// - OXY        : Occidental Petroleum
// 00 ~ 05 : Fix weights & change rebalancing
// 06 ~ 09 : Fix rebalancing & change weights

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[allow(non_snake_case)]
    let TESTNUM = args()
        .nth(1)
        .unwrap_or("0".to_string())
        .parse::<usize>()
        .unwrap();
    let symbols = vec![
        "005930.KS".to_string(),
        // "GLD".to_string(),
        "SPY".to_string(),
        "OXY".to_string(),
    ];
    let symbols = symbols
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let from = "2016-01-01 00:00:00 +09";
    let to = "2023-10-14 00:00:00 +09";
    let init_balance = 1000_0000f64;
    let interest_rate = 0.04f64;
    let weights = vec![
        vec![0.4f64, 0.2, 0.4],
        vec![0.5f64, 0.25, 0.25],  // 6
        vec![0.25f64, 0.5, 0.25],  // 7
        vec![0.25f64, 0.25, 0.5],  // 8
        vec![0.25f64, 0.25, 0.25], // 9
        vec![0.05f64, 0.05, 0.05], // 10
    ];
    let weights = if TESTNUM < 6 {
        weights[0].clone()
    } else {
        weights[TESTNUM - 5].clone()
    };
    let sec_fees = vec![0.00015f64, 0.001, 0.001];
    let sec_fee = weights.dot(&sec_fees);
    let weights = HashMap::from_iter(symbols.clone().into_iter().zip(weights));

    // ┌──────────────────────────────────────────────────────────┐
    //  1. Periodic Rebalancing
    // └──────────────────────────────────────────────────────────┘
    let rebalance_period = if TESTNUM < 6 { PERIOD[TESTNUM] } else { 240 };
    let periodic_rebalance = PeriodicRebalance::new(rebalance_period);
    let bnh_periodic = BuyAndHold::new(weights.clone(), Box::new(periodic_rebalance));
    let mut bt_periodic = Backtester::new(
        &symbols,
        init_balance,
        Box::new(bnh_periodic),
        from,
        to,
        interest_rate,
        sec_fee,
    )
    .await?;
    let report_periodic = bt_periodic.run(120);
    let df_periodic = report_periodic.to_dataframe();
    let dg_periodic = report_periodic.to_report();

    // ┌──────────────────────────────────────────────────────────┐
    //  2. Threshold Rebalancing
    // └──────────────────────────────────────────────────────────┘
    let threshold = if TESTNUM < 6 {
        THRESHOLD[TESTNUM]
    } else {
        5f64
    };
    let threshold_rebalance = ThresholdRebalance::new(threshold, &weights);
    let bnh_threshold = BuyAndHold::new(weights.clone(), Box::new(threshold_rebalance));
    let mut bt_threshold = Backtester::new(
        &symbols,
        init_balance,
        Box::new(bnh_threshold),
        from,
        to,
        interest_rate,
        sec_fee,
    )
    .await?;
    let report_threshold = bt_threshold.run(120);
    let df_threshold = report_threshold.to_dataframe();
    let dg_threshold = report_threshold.to_report();

    df_threshold.print();
    dg_periodic.print();
    dg_threshold.print();

    df_periodic.write_parquet(
        &format!("./data/rebalance_test_periodic_{:02}.parquet", TESTNUM),
        CompressionOptions::Uncompressed,
    )?;
    dg_periodic.write_parquet(
        &format!("./data/rebalance_report_periodic_{:02}.parquet", TESTNUM),
        CompressionOptions::Uncompressed,
    )?;

    df_threshold.write_parquet(
        &format!("./data/rebalance_test_threshold_{:02}.parquet", TESTNUM),
        CompressionOptions::Uncompressed,
    )?;
    dg_threshold.write_parquet(
        &format!("./data/rebalance_report_threshold_{:02}.parquet", TESTNUM),
        CompressionOptions::Uncompressed,
    )?;

    Ok(())
}

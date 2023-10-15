use fmp::strategy::{BuyAndHold, PeriodicRebalance};
use fmp::trade::Backtester;
use peroxide::fuga::*;
use std::collections::HashMap;

// Rebalancing in 60 days
// 0  : Semiconductor
// 00 : Samsung Electronics 100%
// 01 : SK Hynix 100%
// 02 : Samsung Electronics 50% + SK Hynix 50% (No rebalancing)
// 03 : Samsung Electronics 50% + SK Hynix 50%
// 04 : Samsung Electronics 40% + SK Hynix 40% + Cash 20% (No rebalancing)
// 05 : Samsung Electronics 40% + SK Hynix 40% + Cash 20%
// 06 : Samsung Electronics 30% + SK Hynix 30% + Cash 40%
// 07 : Start with (25%, 25%, 50%) -> (45%, 45%, 10%) : Reduce cash ratio
// 1  : 10 representative stocks in each sectors (KOSPI 200)
// List :   Samsung Electronics, LG Chem, POSCO Holdings, Doosan Enerbility, Samsung C&T,
//          SKT, Asia Paper Manufacturing, Samsung Life, NAVER, SsangYong C&E
// 10 : Equal weights
// 11 : More Technology (Tech (Samsung, LG, SKT, NAVER) 16% each, other 6% each)
// 12 : Less Technology (Tech (Samsung, LG, SKT, NAVER) 4% each, other 14% each)

const TESTNUM: usize = 7;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let symbols = vec!["005930.KS".to_string()];
    // let symbols = vec!["000660.KS".to_string()];
    let symbols = vec!["005930.KS".to_string(), "086520.KQ".to_string()];
    //let symbols = vec![
    //    "005930.KS",    // Samsung Electronics// //
    //    "051910.KS",    // LG Chem
    //    "005490.KS",    // POSCO Holdings
    //    "034020.KS",    // Doosan Enerbility
    //    "028260.KS",    // Samsung C&T
    //    "017670.KS",    // SKT
    //    "002310.KS",    // Asia Paper Manufacturing
    //    "032830.KS",    // Samsung Life
    //    "035420.KS",    // NAVER
    //    "003410.KS",    // SsangYong C&E
    //];
    let symbols = symbols
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let from = "2019-03-08 00:00:00 +09";
    let to = "2023-10-12 00:00:00 +09";
    let init_balance = 1000_0000f64;
    let interest_rate = 0.04f64;
    // let sec_fee = 0.00015f64;
    let sec_fee = {
        // 0.00015 * 0.8 + 0.001 * 0.2
        0.00015
    };

    // let weight_vec = vec![1f64];
    let weight_vec = vec![0.4f64, 0.4];
    // let weight_vec = vec![0.4f64, 0.4];
    // let weight_vec = vec![0.1f64; symbols.len()];
    //let weight_vec = vec![
    //    0.16f64, 0.16, 0.06, 0.06, 0.06, 0.16, 0.06, 0.06, 0.16, 0.06
    //];
    let weight = HashMap::from_iter(symbols.clone().into_iter().zip(weight_vec));
    let rebalance_period = 60;
    let periodic_rebalance = PeriodicRebalance::new(rebalance_period);

    let bnh = BuyAndHold::new(weight, Box::new(periodic_rebalance));
    let mut bt = Backtester::new(
        &symbols,
        init_balance,
        Box::new(bnh),
        from,
        to,
        interest_rate,
        sec_fee,
    )
    .await?;

    let report = bt.run(120);
    let df = report.to_dataframe();
    let dg = report.to_report();

    df.print();
    dg.print();

    df.write_parquet(
        //"./data/trade_test_00.parquet",
        &format!("./data/trade_test_{:02}.parquet", TESTNUM),
        CompressionOptions::Uncompressed,
    )?;
    dg.write_parquet(
        //"./data/trade_report_00.parquet",
        &format!("./data/trade_report_{:02}.parquet", TESTNUM),
        CompressionOptions::Uncompressed,
    )?;

    Ok(())
}

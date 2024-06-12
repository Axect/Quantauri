use chrono::NaiveDate;
use peroxide::fuga::*;
use quantauri::api::download_stocks;

#[tokio::main]
async fn main() {
    let symbols = vec![
        "005930.KS".to_string(),
        "005490.KS".to_string(),
    ];
    let from = NaiveDate::from_ymd_opt(2012,1,1).unwrap();
    let to = NaiveDate::from_ymd_opt(2024,1,1).unwrap();

    let resp_vec = download_stocks(&symbols, from, to).await.unwrap();
    let mut df_vec = vec![];
    for resp in resp_vec {
        let df = resp.to_dataframe();
        df_vec.push(df);
    }

    for df in df_vec {
        df.print();
    }
}

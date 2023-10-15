use peroxide::fuga::*;
use quantauri::api::download_stocks;

#[tokio::main]
async fn main() {
    let symbols = vec![
        "005930.KS".to_string(),
        "005490.KS".to_string(),
        "^TNX".to_string(),
    ];
    let from = "2018-01-01 00:00:00 +09";
    let to = "2023-10-12 00:00:00 +09";

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

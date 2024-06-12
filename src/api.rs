use peroxide::fuga::*;
use stock_data::*;
use chrono::NaiveDate;

pub async fn download_stocks(
    symbols: &[String],
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<HistoricalChart>, Box<dyn std::error::Error>> {
    let mut hist_vec = vec![];
    for symbol in symbols {
        let url = build_yahoo_finance_url_from_dates(symbol, from, to, "1d", true);
        println!("Downloading {}\tfrom {} to {}", symbol, from, to);

        let mut bytes = vec![];
        while bytes.is_empty() {
            bytes = match download_stock_data(&url).await {
                Ok(b) => b,
                Err(e) => {
                    println!("Error: {}", e);
                    vec![]
                }
            };
        }

        let data_dir = "data";
        if !std::path::Path::new(data_dir).exists() {
            std::fs::create_dir(data_dir)?;
        }
        let path = format!("{}/{}", data_dir, symbol);
        let temp_file = path + ".csv";
        write_stock_data(&bytes, &temp_file).await?;

        // Wait for file to be written
        // Check file size > 0 bytes
        while std::fs::metadata(&temp_file)?.len() == 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let mut df = DataFrame::read_csv(&temp_file, ',')?;

        // Find "null" in the data
        df["Open"].mut_map(|x: &mut String| if x == "null" { *x = "0.0".to_string(); });
        df["High"].mut_map(|x: &mut String| if x == "null" { *x = "0.0".to_string(); });
        df["Low"].mut_map(|x: &mut String| if x == "null" { *x = "0.0".to_string(); });
        df["Close"].mut_map(|x: &mut String| if x == "null" { *x = "0.0".to_string(); });
        df["Adj Close"].mut_map(|x: &mut String| if x == "null" { *x = "0.0".to_string(); });
        df["Volume"].mut_map(|x: &mut String| if x == "null" { *x = "0".to_string(); });
        df.as_types(vec![Str, F64, F64, F64, F64, F64, U64]);

        hist_vec.push(df.to_historical_chart(symbol));
    }
    Ok(hist_vec)
}

#[derive(Debug, Clone, Copy)]
pub struct Chart {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub adj_close: f64,
}

impl Chart {
    pub fn get_open(&self) -> f64 {
        self.open
    }

    pub fn get_high(&self) -> f64 {
        self.high
    }

    pub fn get_low(&self) -> f64 {
        self.low
    }

    pub fn get_close(&self) -> f64 {
        self.close
    }

    pub fn get_volume(&self) -> u64 {
        self.volume
    }

    pub fn get_adj_close(&self) -> f64 {
        self.adj_close
    }
}

#[derive(Debug, Clone)]
pub struct HistoricalChart {
    pub symbol: String,
    pub date: Vec<String>,
    pub chart: Vec<Chart>,
}

impl HistoricalChart {
    pub fn get_dates(&self) -> &Vec<String> {
        &self.date
    }

    pub fn get_charts(&self) -> &Vec<Chart> {
        &self.chart
    }

    pub fn get_open_vec(&self) -> Vec<f64> {
        self.chart.iter().map(|x| x.get_open()).collect::<Vec<f64>>()
    }

    pub fn get_high_vec(&self) -> Vec<f64> {
        self.chart.iter().map(|x| x.get_high()).collect::<Vec<f64>>()
    }

    pub fn get_low_vec(&self) -> Vec<f64> {
        self.chart.iter().map(|x| x.get_low()).collect::<Vec<f64>>()
    }

    pub fn get_close_vec(&self) -> Vec<f64> {
        self.chart.iter().map(|x| x.get_close()).collect::<Vec<f64>>()
    }

    pub fn get_volume_vec(&self) -> Vec<u64> {
        self.chart.iter().map(|x| x.get_volume()).collect::<Vec<u64>>()
    }

    pub fn get_adj_close_vec(&self) -> Vec<f64> {
        self.chart.iter().map(|x| x.get_adj_close()).collect::<Vec<f64>>()
    }

    pub fn to_dataframe(&self) -> DataFrame {
        let mut df = DataFrame::new(vec![]);
        df.push("date", Series::new(self.get_dates().clone()));
        df.push("open", Series::new(self.get_open_vec()));
        df.push("high", Series::new(self.get_high_vec()));
        df.push("low", Series::new(self.get_low_vec()));
        df.push("close", Series::new(self.get_close_vec()));
        df.push("volume", Series::new(self.get_volume_vec()));
        df.push("adj_close", Series::new(self.get_adj_close_vec()));
        df
    }
}

pub trait Quote {
    fn get_date(&self) -> Vec<String>;
    fn get_open(&self) -> Vec<f64>;
    fn get_high(&self) -> Vec<f64>;
    fn get_low(&self) -> Vec<f64>;
    fn get_close(&self) -> Vec<f64>;
    fn get_volume(&self) -> Vec<u64>;
    fn get_adj_close(&self) -> Vec<f64>;
    fn to_dataframe(&self) -> DataFrame;
    fn to_chart_vec(&self) -> Vec<Chart>;
    fn to_historical_chart(&self, symbol: &str) -> HistoricalChart;
}

impl Quote for DataFrame {
    fn get_date(&self) -> Vec<String> {
        self["Date"].to_vec()
    }

    fn get_open(&self) -> Vec<f64> {
        self["Open"].to_vec()
    }

    fn get_high(&self) -> Vec<f64> {
        self["High"].to_vec()
    }

    fn get_low(&self) -> Vec<f64> {
        self["Low"].to_vec()
    }

    fn get_close(&self) -> Vec<f64> {
        self["Close"].to_vec()
    }

    fn get_volume(&self) -> Vec<u64> {
        self["Volume"].to_vec()
    }

    fn get_adj_close(&self) -> Vec<f64> {
        self["Adj Close"].to_vec()
    }

    fn to_dataframe(&self) -> DataFrame {
        self.clone()
    }

    fn to_chart_vec(&self) -> Vec<Chart> {
        let date = self.get_date();
        let open = self.get_open();
        let high = self.get_high();
        let low = self.get_low();
        let close = self.get_close();
        let volume = self.get_volume();
        let adj_close = self.get_adj_close();

        date.iter()
            .zip(open.iter())
            .zip(high.iter())
            .zip(low.iter())
            .zip(close.iter())
            .zip(volume.iter())
            .zip(adj_close.iter())
            .map(|((((((_, o), h), l), c), v), a)| Chart {
                open: *o,
                high: *h,
                low: *l,
                close: *c,
                volume: *v,
                adj_close: *a,
            })
            .collect()
    }

    fn to_historical_chart(&self, symbol: &str) -> HistoricalChart {
        let date = self.get_date();
        let chart = self.to_chart_vec();
        HistoricalChart {
            symbol: symbol.to_string(),
            date,
            chart,
        }
    }
}

//impl Quote for YResponse {
//    fn get_timestemp(&self) -> Vec<String> {
//        let quotes = self.quotes().unwrap();
//        // Change u64 -> String (YYYY-MM-DD) with datetime
//        quotes
//            .into_iter()
//            .map(|x| {
//                OffsetDateTime::from_unix_timestamp(x.timestamp as i64)
//                    .unwrap()
//                    .date()
//                    .to_string()
//            })
//            .collect()
//    }
//
//    fn get_open(&self) -> Vec<f64> {
//        let quotes = self.quotes().unwrap();
//        quotes.into_iter().map(|x| x.open).collect()
//    }
//
//    fn get_high(&self) -> Vec<f64> {
//        let quotes = self.quotes().unwrap();
//        quotes.into_iter().map(|x| x.high).collect()
//    }
//
//    fn get_low(&self) -> Vec<f64> {
//        let quotes = self.quotes().unwrap();
//        quotes.into_iter().map(|x| x.low).collect()
//    }
//
//    fn get_close(&self) -> Vec<f64> {
//        let quotes = self.quotes().unwrap();
//        quotes.into_iter().map(|x| x.close).collect()
//    }
//
//    fn get_volume(&self) -> Vec<u64> {
//        let quotes = self.quotes().unwrap();
//        quotes.into_iter().map(|x| x.volume).collect()
//    }
//
//    fn get_adj_close(&self) -> Vec<f64> {
//        let quotes = self.quotes().unwrap();
//        quotes.into_iter().map(|x| x.adjclose).collect()
//    }
//
//    fn to_dataframe(&self) -> DataFrame {
//        let mut df = DataFrame::new(vec![]);
//        df.push("timestamp", Series::new(self.get_timestemp()));
//        df.push("open", Series::new(self.get_open()));
//        df.push("high", Series::new(self.get_high()));
//        df.push("low", Series::new(self.get_low()));
//        df.push("close", Series::new(self.get_close()));
//        df.push("volume", Series::new(self.get_volume()));
//        df.push("adjclose", Series::new(self.get_adj_close()));
//        df
//    }
//
//    fn to_chart_vec(&self) -> Vec<Chart> {
//        let quotes = self.quotes().unwrap();
//        quotes
//            .into_iter()
//            .map(|x| Chart {
//                open: x.open,
//                high: x.high,
//                low: x.low,
//                close: x.close,
//                volume: x.volume,
//                adj_close: x.adjclose,
//            })
//            .collect()
//    }
//
//    fn to_historical_chart(&self, symbol: &str) -> HistoricalChart {
//        let date = self.get_timestemp();
//        let chart = self.to_chart_vec();
//        HistoricalChart {
//            symbol: symbol.to_string(),
//            date,
//            chart,
//        }
//    }
//}

//use serde::Deserialize;
//use reqwest::header::*;
//pub fn download_stocks(symbols: &[String], from: &str, to: &str, api_key: &str) -> Result<HistoricalStockList, Box<dyn std::error::Error>> {
//    let mut hpf_vec = vec![];
//    for symbol in symbols {
//        let mut hpf = HistoricalPriceFull::new(symbol);
//        hpf.download_interval(api_key, from, to)?;
//        hpf_vec.push(hpf);
//    }
//    Ok(HistoricalStockList {
//        historicalStockList: hpf_vec
//    })
//}
//
//pub fn download_risk_free(from: &str, to: &str, api_key: &str) -> (Vec<String>, Vec<f64>) {
//    let symbol = "^TNX";
//    let mut hpf = HistoricalPriceFull::new(symbol);
//    hpf.download_interval(api_key, from, to).unwrap();
//    let tnx = hpf.get_close_vec();
//    let mut risk_free = tnx.fmap(|x| (1f64 * x / 100f64).powf(1f64 / 252f64) - 1f64);
//    risk_free[0 .. 120].fill(0f64);
//    (hpf.get_date_vec(), risk_free)
//}
//
//#[allow(non_snake_case)]
//#[derive(Debug, Deserialize)]
//pub struct HistoricalStockList {
//    historicalStockList: Vec<HistoricalPriceFull>,
//}
//
//impl HistoricalStockList {
//    pub fn get_historical_price_full(&self, index: usize) -> &HistoricalPriceFull {
//        &self.historicalStockList[index]
//    }
//}
//
//
//#[derive(Debug, Deserialize)]
//pub struct HistoricalPriceFull {
//    symbol: String,
//    historical: Vec<HistoricalPrice>,
//}
//
//#[allow(non_snake_case)]
//#[derive(Debug, Deserialize, Clone)]
//pub struct HistoricalPrice {
//    date: String,
//    open: f64,
//    high: f64,
//    low: f64,
//    close: f64,
//    adjClose: f64,
//    volume: f64,
//    unadjustedVolume: f64,
//    change: f64,
//    changePercent: f64,
//    vwap: f64,
//    label: String,
//    changeOverTime: f64,
//}
//
//#[derive(Debug, Deserialize)]
//pub struct RSI {
//    date: String,
//    open: f64,
//    high: f64,
//    low: f64,
//    close: f64,
//    volume: f64,
//    rsi: f64,
//}
//
//#[derive(Debug, Deserialize)]
//pub struct DailyRSI {
//    symbol: String,
//    rsi: Vec<RSI>,
//}
//
//#[derive(Debug, Deserialize)]
//pub struct Treasury {
//    date: String,
//    month1: f64,
//    month2: f64,
//    month3: f64,
//    month6: f64,
//    year1: f64,
//    year2: f64,
//    year3: f64,
//    year5: f64,
//    year7: f64,
//    year10: f64,
//    year20: f64,
//    year30: f64,
//}
//
//#[derive(Debug, Deserialize)]
//pub struct DailyTreasury {
//    treasury: Vec<Treasury>,
//}
//
//impl Treasury {
//    pub fn get_date(&self) -> &str {
//        &self.date
//    }
//
//    pub fn get_month1(&self) -> f64 {
//        self.month1
//    }
//
//    pub fn get_month2(&self) -> f64 {
//        self.month2
//    }
//
//    pub fn get_month3(&self) -> f64 {
//        self.month3
//    }
//
//    pub fn get_month6(&self) -> f64 {
//        self.month6
//    }
//
//    pub fn get_year1(&self) -> f64 {
//        self.year1
//    }
//
//    pub fn get_year2(&self) -> f64 {
//        self.year2
//    }
//
//    pub fn get_year3(&self) -> f64 {
//        self.year3
//    }
//
//    pub fn get_year5(&self) -> f64 {
//        self.year5
//    }
//
//    pub fn get_year7(&self) -> f64 {
//        self.year7
//    }
//
//    pub fn get_year10(&self) -> f64 {
//        self.year10
//    }
//
//    pub fn get_year20(&self) -> f64 {
//        self.year20
//    }
//
//    pub fn get_year30(&self) -> f64 {
//        self.year30
//    }
//}
//
//impl DailyTreasury {
//    pub fn new() -> Self {
//        Self {
//            treasury: vec![],
//        }
//    }
//
//    pub fn download_full(&mut self, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
//        let base_url = "https://financialmodelingprep.com/api/v4/treasury";
//        let url = format!("{}/?apikey={}", base_url, api_key);
//
//        let mut headers = HeaderMap::new();
//        headers.insert(
//            UPGRADE_INSECURE_REQUESTS,
//            "1".parse()?,
//        );
//
//        let client = reqwest::blocking::Client::builder()
//            .default_headers(headers)
//            .build()?;
//
//        let resp = client.get(&url).send()?.json::<Vec<Treasury>>()?;
//        self.treasury = resp;
//
//        Ok(())
//    }
//
//    pub fn download_interval(&mut self, api_key: &str, from: &str, to: &str) -> Result<(), Box<dyn std::error::Error>> {
//        let base_url = "https://financialmodelingprep.com/api/v4/treasury";
//        let url = format!("{}/?from={}&to={}&apikey={}", base_url, from, to, api_key);
//
//        let mut headers = HeaderMap::new();
//        headers.insert(
//            UPGRADE_INSECURE_REQUESTS,
//            "1".parse()?,
//        );
//
//        let client = reqwest::blocking::Client::builder()
//            .default_headers(headers)
//            .build()?;
//
//        let resp = client.get(&url).send()?.json::<Vec<Treasury>>()?;
//        self.treasury = resp;
//
//        Ok(())
//    }
//
//    pub fn get_date_vec(&self) -> Vec<String> {
//        self.treasury.iter().rev().map(|x| x.get_date().to_string()).collect::<Vec<String>>()
//    }
//
//    pub fn get_month1_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_month1()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_month2_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_month2()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_month3_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_month3()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_month6_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_month6()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_year1_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_year1()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_year2_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_year2()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_year3_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_year3()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_year5_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_year5()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_year7_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_year7()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_year10_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_year10()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_year20_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_year20()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_year30_vec(&self) -> Vec<f64> {
//        self.treasury.iter().rev().map(|x| x.get_year30()).collect::<Vec<f64>>()
//    }
//
//    pub fn to_dataframe_full(&self) -> DataFrame {
//        let mut df = DataFrame::new(vec![]);
//        df.push("date", Series::new(self.get_date_vec()));
//        df.push("month1", Series::new(self.get_month1_vec()));
//        df.push("month2", Series::new(self.get_month2_vec()));
//        df.push("month3", Series::new(self.get_month3_vec()));
//        df.push("month6", Series::new(self.get_month6_vec()));
//        df.push("year1", Series::new(self.get_year1_vec()));
//        df.push("year2", Series::new(self.get_year2_vec()));
//        df.push("year3", Series::new(self.get_year3_vec()));
//        df.push("year5", Series::new(self.get_year5_vec()));
//        df.push("year7", Series::new(self.get_year7_vec()));
//        df.push("year10", Series::new(self.get_year10_vec()));
//        df.push("year20", Series::new(self.get_year20_vec()));
//        df.push("year30", Series::new(self.get_year30_vec()));
//        df
//    }
//
//    pub fn to_dataframe_simple(&self) -> DataFrame {
//        let mut df = DataFrame::new(vec![]);
//        df.push("date", Series::new(self.get_date_vec()));
//        df.push("year10", Series::new(self.get_year10_vec()));
//        df
//    }
//}
//
//impl RSI {
//    pub fn get_date(&self) -> &str {
//        // Only get YYYY-MM-DD
//        &self.date[..10]
//    }
//
//    pub fn get_open(&self) -> f64 {
//        self.open
//    }
//
//    pub fn get_high(&self) -> f64 {
//        self.high
//    }
//
//    pub fn get_low(&self) -> f64 {
//        self.low
//    }
//
//    pub fn get_close(&self) -> f64 {
//        self.close
//    }
//
//    pub fn get_volume(&self) -> f64 {
//        self.volume
//    }
//
//    pub fn get_rsi(&self) -> f64 {
//        self.rsi
//    }
//}
//
//impl DailyRSI {
//    pub fn new(symbol: &str) -> Self {
//        Self {
//            symbol: symbol.to_string(),
//            rsi: vec![],
//        }
//    }
//
//    pub fn download_full(&mut self, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
//        let base_url = "https://financialmodelingprep.com/api/v3/technical_indicator/daily/";
//        let url = format!("{}{}?period=14&type=RSI&apikey={}", base_url, self.symbol, api_key);
//
//        let mut headers = HeaderMap::new();
//        headers.insert(
//            UPGRADE_INSECURE_REQUESTS,
//            "1".parse()?,
//        );
//
//        let client = reqwest::blocking::Client::builder()
//            .default_headers(headers)
//            .build()?;
//
//        let resp = client.get(&url).send()?.json::<Vec<RSI>>()?;
//        self.rsi = resp;
//
//        Ok(())
//    }
//
//    pub fn download_interval(&mut self, api_key: &str, from: &str, to: &str) -> Result<(), Box<dyn std::error::Error>> {
//        let base_url = "https://financialmodelingprep.com/api/v3/technical_indicator/daily/";
//        let url = format!("{}{}?from={}&to={}&period=14&type=RSI&apikey={}", base_url, self.symbol, from, to, api_key);
//
//        let mut headers = HeaderMap::new();
//        headers.insert(
//            UPGRADE_INSECURE_REQUESTS,
//            "1".parse()?,
//        );
//
//        let client = reqwest::blocking::Client::builder()
//            .default_headers(headers)
//            .build()?;
//
//        let resp = client.get(&url).send()?.json::<Vec<RSI>>()?;
//        self.rsi = resp;
//
//        Ok(())
//    }
//
//    pub fn get_symbol(&self) -> &str {
//        &self.symbol
//    }
//
//    pub fn get_date_vec(&self) -> Vec<String> {
//        self.rsi.iter().rev().map(|x| x.get_date().to_string()).collect::<Vec<String>>()
//    }
//
//    pub fn get_open_vec(&self) -> Vec<f64> {
//        self.rsi.iter().rev().map(|x| x.get_open()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_high_vec(&self) -> Vec<f64> {
//        self.rsi.iter().rev().map(|x| x.get_high()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_low_vec(&self) -> Vec<f64> {
//        self.rsi.iter().rev().map(|x| x.get_low()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_close_vec(&self) -> Vec<f64> {
//        self.rsi.iter().rev().map(|x| x.get_close()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_volume_vec(&self) -> Vec<f64> {
//        self.rsi.iter().rev().map(|x| x.get_volume()).collect::<Vec<f64>>()
//    }
//
//    pub fn get_rsi_vec(&self) -> Vec<f64> {
//        self.rsi.iter().rev().map(|x| x.get_rsi()).collect::<Vec<f64>>()
//    }
//
//    pub fn to_dataframe(&self) -> DataFrame {
//        let mut df = DataFrame::new(vec![]);
//        df.push("date", Series::new(self.get_date_vec()));
//        df.push("open", Series::new(self.get_open_vec()));
//        df.push("high", Series::new(self.get_high_vec()));
//        df.push("low", Series::new(self.get_low_vec()));
//        df.push("close", Series::new(self.get_close_vec()));
//        df.push("volume", Series::new(self.get_volume_vec()));
//        df.push("rsi", Series::new(self.get_rsi_vec()));
//
//        df
//    }
//}
//
//impl HistoricalPrice {
//    pub fn new() -> Self {
//        Self {
//            date: "".to_string(),
//            open: 0f64,
//            high: 0f64,
//            low: 0f64,
//            close: 0f64,
//            adjClose: 0f64,
//            volume: 0f64,
//            unadjustedVolume: 0f64,
//            change: 0f64,
//            changePercent: 0f64,
//            vwap: 0f64,
//            label: "".to_string(),
//            changeOverTime: 0f64,
//        }
//    }
//
//    pub fn get_date(&self) -> &str {
//        &self.date
//    }
//
//    pub fn get_open(&self) -> f64 {
//        self.open
//    }
//
//    pub fn get_high(&self) -> f64 {
//        self.high
//    }
//
//    pub fn get_low(&self) -> f64 {
//        self.low
//    }
//
//    pub fn get_close(&self) -> f64 {
//        self.close
//    }
//
//    pub fn get_adj_close(&self) -> f64 {
//        self.adjClose
//    }
//
//    pub fn get_volume(&self) -> f64 {
//        self.volume
//    }
//
//    pub fn get_unadjusted_volume(&self) -> f64 {
//        self.unadjustedVolume
//    }
//
//    pub fn get_change(&self) -> f64 {
//        self.change
//    }
//
//    pub fn get_change_percent(&self) -> f64 {
//        self.changePercent
//    }
//
//    pub fn get_vwap(&self) -> f64 {
//        self.vwap
//    }
//
//    pub fn get_label(&self) -> &str {
//        &self.label
//    }
//
//    pub fn get_change_over_time(&self) -> f64 {
//        self.changeOverTime
//    }
//}
//
//#[allow(dead_code)]
//impl HistoricalPriceFull {
//    pub fn new(symbol: &str) -> Self {
//        Self {
//            symbol: symbol.to_string(),
//            historical: vec![],
//        }
//    }
//
//    pub fn download_full(&mut self, api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
//        let base_url = "https://financialmodelingprep.com/api/v3/historical-price-full/";
//        let url = format!("{}{}?apikey={}", base_url, self.symbol, api_key);
//
//        let mut headers = HeaderMap::new();
//        headers.insert(
//            UPGRADE_INSECURE_REQUESTS,
//            "1".parse()?,
//        );
//
//        let client = reqwest::blocking::Client::builder()
//            .default_headers(headers)
//            .build()?;
//
//        let resp = client.get(&url).send()?.json::<HistoricalPriceFull>()?;
//        self.historical = resp.historical;
//
//        Ok(())
//    }
//
//    pub fn download_interval(&mut self, api_key: &str, from: &str, to: &str) -> Result<(), Box<dyn std::error::Error>> {
//        let base_url = "https://financialmodelingprep.com/api/v3/historical-price-full/";
//        let url = format!("{}{}?from={}&to={}&apikey={}", base_url, self.symbol, from, to, api_key);
//
//        let mut headers = HeaderMap::new();
//        headers.insert(
//            UPGRADE_INSECURE_REQUESTS,
//            "1".parse()?,
//        );
//
//        let client = reqwest::blocking::Client::builder()
//            .default_headers(headers)
//            .build()?;
//
//        let resp = client.get(&url).send()?.json::<HistoricalPriceFull>()?;
//        self.historical = resp.historical;
//
//        Ok(())
//    }
//
//    pub fn get_symbol(&self) -> &str {
//        &self.symbol
//    }
//
//    pub fn get_historical(&self) -> &Vec<HistoricalPrice> {
//        &self.historical
//    }
//
//    pub fn get_mut_historical(&mut self) -> &mut Vec<HistoricalPrice> {
//        &mut self.historical
//    }
//
//    pub fn get_date_vec(&self) -> Vec<String> {
//        self.historical.iter().map(|x| x.date.clone()).rev().collect()
//    }
//
//    pub fn get_open_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.open).rev().collect()
//    }
//
//    pub fn get_high_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.high).rev().collect()
//    }
//
//    pub fn get_low_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.low).rev().collect()
//    }
//
//    pub fn get_close_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.close).rev().collect()
//    }
//
//    pub fn get_adj_close_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.adjClose).rev().collect()
//    }
//
//    pub fn get_volume_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.volume).rev().collect()
//    }
//
//    pub fn get_unadjusted_volume_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.unadjustedVolume).rev().collect()
//    }
//
//    pub fn get_change_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.change).rev().collect()
//    }
//
//    pub fn get_change_percent_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.changePercent).rev().collect()
//    }
//
//    pub fn get_vwap_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.vwap).rev().collect()
//    }
//
//    pub fn get_label_vec(&self) -> Vec<String> {
//        self.historical.iter().map(|x| x.label.clone()).rev().collect()
//    }
//
//    pub fn get_change_over_time_vec(&self) -> Vec<f64> {
//        self.historical.iter().map(|x| x.changeOverTime).rev().collect()
//    }
//
//    pub fn to_dataframe_full(&self) -> DataFrame {
//        let mut df = DataFrame::new(vec![]);
//        df.push("date", Series::new(self.get_date_vec()));
//        df.push("open", Series::new(self.get_open_vec()));
//        df.push("high", Series::new(self.get_high_vec()));
//        df.push("low", Series::new(self.get_low_vec()));
//        df.push("close", Series::new(self.get_close_vec()));
//        df.push("adjClose", Series::new(self.get_adj_close_vec()));
//        df.push("volume", Series::new(self.get_volume_vec()));
//        df.push("unadjustedVolume", Series::new(self.get_unadjusted_volume_vec()));
//        df.push("change", Series::new(self.get_change_vec()));
//        df.push("changePercent", Series::new(self.get_change_percent_vec()));
//        df.push("vwap", Series::new(self.get_vwap_vec()));
//        df.push("label", Series::new(self.get_label_vec()));
//        df.push("changeOverTime", Series::new(self.get_change_over_time_vec()));
//        df
//    }
//
//    pub fn to_dataframe_simple(&self) -> DataFrame {
//        let mut df = DataFrame::new(vec![]);
//        df.push("date", Series::new(self.get_date_vec()));
//        df.push("open", Series::new(self.get_open_vec()));
//        df.push("high", Series::new(self.get_high_vec()));
//        df.push("low", Series::new(self.get_low_vec()));
//        df.push("close", Series::new(self.get_close_vec()));
//        df.push("volume", Series::new(self.get_volume_vec()));
//        df
//    }
//}

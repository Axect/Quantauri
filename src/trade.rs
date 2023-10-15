use crate::api::{download_stocks, Chart};
use crate::strategy::Strategy;
use peroxide::fuga::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Portfolio {
    pub balance: f64,
    pub shares: HashMap<String, usize>,
}

impl Portfolio {
    pub fn new(balance: f64, symbols: &[String]) -> Self {
        let mut shares = HashMap::new();
        for symbol in symbols.iter() {
            shares.insert(symbol.to_string(), 0);
        }
        Self { balance, shares }
    }

    pub fn get_symbols(&self) -> Vec<String> {
        self.shares.keys().cloned().collect::<Vec<String>>()
    }

    pub fn get_share(&self, symbol: &str) -> Option<usize> {
        self.shares.get(symbol).cloned()
    }

    pub fn get_share_mut(&mut self, symbol: &str) -> Option<&mut usize> {
        self.shares.get_mut(symbol)
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn update_share(&mut self, symbol: &str, share: usize) {
        self.shares.insert(symbol.to_string(), share);
    }

    pub fn update_balance(&mut self, balance: f64) {
        self.balance = balance;
    }
}

#[derive(Debug, Clone)]
pub struct Order {
    pub symbol: String,
    pub shares: isize, // positive for buy, negative for sell
}

impl Order {
    pub fn new(symbol: &str, shares: isize) -> Self {
        Self {
            symbol: symbol.to_string(),
            shares,
        }
    }
}

pub struct Backtester {
    pub portfolio: Portfolio,
    pub strategy: Box<dyn Strategy>,
    pub market_data: MarketData,
    pub interest_rate: f64,
    pub sec_fee: f64,
}

pub struct BacktestReport {
    pub portfolio: Portfolio,
    pub strategy: String,
    pub market_data: MarketData,
    pub daily_return: Vec<f64>,
    pub cumulative_return: Vec<f64>,
    pub rolling_volatility: Vec<f64>,
    pub rolling_sharpe_ratio: Vec<f64>,
    pub drawdown: Vec<f64>,
    pub cagr: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub mdd: f64,
    pub balance_history: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct MarketData {
    pub from: String,
    pub to: String,
    pub date: Vec<String>,
    pub chart: Vec<HashMap<String, Chart>>,
    pub risk_free: Vec<f64>,
}

impl MarketData {
    pub async fn new(
        symbols: &[String],
        from: &str,
        to: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let vec_hist = download_stocks(symbols, from, to).await?;
        let mut chart_vec_map = HashMap::new();
        let mut date_zip = vec![];
        for (symbol, hist) in symbols.iter().zip(vec_hist.iter()) {
            date_zip.push(
                hist.get_dates()
                    .iter()
                    .cloned()
                    .enumerate()
                    .collect::<Vec<(usize, String)>>(),
            );
            chart_vec_map.insert(symbol.to_string(), hist.get_charts().clone());
        }
        let risk_free_ticker = vec!["^TNX".to_string()];
        let risk_free = download_stocks(&risk_free_ticker, from, to).await?;
        let risk_free = risk_free[0].clone();
        let date_risk_free = risk_free.get_dates();
        let risk_free = risk_free.get_charts();
        let risk_free_zip = date_risk_free
            .iter()
            .cloned()
            .enumerate()
            .collect::<Vec<(usize, String)>>();

        // Check date
        // - There are some missing dates
        // - Only accept common dates
        // - To do this, we need to make dates to HashSet
        let date_vec = date_zip
            .iter()
            .map(|x| x.iter().map(|y| y.1.clone()).collect::<Vec<String>>())
            .collect::<Vec<Vec<String>>>();
        let mut date_set = date_vec[0].iter().cloned().collect::<HashSet<String>>();
        for date_ in date_vec.iter().skip(1) {
            let date_set_2 = date_.iter().cloned().collect::<HashSet<String>>();
            date_set.retain(|x| date_set_2.contains(x));
        }
        let risk_free_set = date_risk_free.iter().cloned().collect::<HashSet<String>>();
        date_set.retain(|x| risk_free_set.contains(x));
        let mut date = date_set.iter().cloned().collect::<Vec<String>>();
        date.sort();

        // Remove data correspond to deleted dates in hp_map
        let idx_vec = date_zip
            .iter()
            .map(|x| {
                x.iter()
                    .enumerate()
                    .filter(|(_, y)| date_set.contains(&y.1))
                    .map(|(i, _)| i)
                    .collect::<Vec<usize>>()
            })
            .collect::<Vec<Vec<usize>>>();

        for (symbol, idx) in symbols.iter().zip(idx_vec.iter()) {
            let chart_vec = chart_vec_map.get_mut(symbol).unwrap();
            let mut hist_new = vec![];
            for &j in idx {
                hist_new.push(chart_vec[j]);
            }
            *chart_vec = hist_new;
        }

        let idx_vec = risk_free_zip
            .iter()
            .filter(|&x| date_set.contains(&x.1))
            .map(|x| x.0)
            .collect::<Vec<usize>>();
        let mut risk_free_new = vec![];
        for i in idx_vec {
            risk_free_new.push(risk_free[i].get_close());
        }
        //let risk_free = risk_free_new;
        //let risk_free = {
        //    let mut result = vec![0f64; risk_free.len()];
        //    for i in 1..risk_free.len() {
        //        result[i] = (risk_free[i] - risk_free[i - 1]) / risk_free[i - 1];
        //    }
        //    result
        //};
        // Annual rate -> Daily rate
        let risk_free = risk_free_new.fmap(|x| (1f64 + x / 100f64).powf(1f64 / 252f64) - 1f64);

        assert_eq!(date.len(), risk_free.len());

        let mut chart_vec_new = vec![HashMap::new(); risk_free.len()];
        for (i, chart_new) in chart_vec_new.iter_mut().enumerate() {
            for symbol in symbols.iter() {
                let chart = &chart_vec_map.get(symbol).as_ref().unwrap()[i];
                chart_new.insert(
                    symbol.to_string(),
                    Chart {
                        open: chart.get_open(),
                        high: chart.get_high(),
                        low: chart.get_low(),
                        close: chart.get_close(),
                        volume: chart.get_volume(),
                        adj_close: chart.get_adj_close(),
                    },
                );
            }
        }

        Ok(Self {
            from: from.to_string(),
            to: to.to_string(),
            date,
            chart: chart_vec_new,
            risk_free,
        })
    }

    pub fn len(&self) -> usize {
        self.chart.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chart.is_empty()
    }
}

impl Backtester {
    pub async fn new(
        symbols: &[String],
        init_balance: f64,
        strategy: Box<dyn Strategy>,
        from: &str,
        to: &str,
        interest_rate: f64,
        sec_fee: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let market_data = MarketData::new(symbols, from, to).await?;
        let portfolio = Portfolio::new(init_balance, symbols);
        Ok(Self {
            portfolio,
            strategy,
            market_data,
            interest_rate,
            sec_fee,
        })
    }

    pub fn get_symbols(&self) -> Vec<String> {
        self.portfolio.get_symbols()
    }

    pub fn get_balance(&self) -> f64 {
        self.portfolio.balance
    }

    pub fn get_share(&self, symbol: &str) -> Option<usize> {
        self.portfolio.get_share(symbol)
    }

    pub fn get_share_mut(&mut self, symbol: &str) -> Option<&mut usize> {
        self.portfolio.get_share_mut(symbol)
    }

    pub fn update_share(&mut self, symbol: &str, share: usize) {
        self.portfolio.shares.insert(symbol.to_string(), share);
    }

    pub fn update_balance(&mut self, balance: f64) {
        self.portfolio.update_balance(balance)
    }

    pub fn get_chart(&self, timestamp: usize) -> &HashMap<String, Chart> {
        &self.market_data.chart[timestamp - 1]
    }

    pub fn get_risk_free(&self) -> &Vec<f64> {
        &self.market_data.risk_free
    }

    pub fn get_interest_rate(&self) -> f64 {
        self.interest_rate
    }

    pub fn execute_order(&mut self, order: &HashMap<String, Order>, timestamp: usize) {
        let chart = self.get_chart(timestamp).clone();
        for (symbol, o) in order.iter() {
            let balance = self.get_balance();
            let price = chart.get(symbol).as_ref().unwrap().adj_close;
            let current_share = self.get_share(symbol).unwrap();
            let order_share = o.shares;
            let cost = price * (order_share as f64) * (1f64 + self.sec_fee);
            self.update_balance(balance - cost);
            self.update_share(symbol, (current_share as isize + order_share) as usize);
        }
    }

    pub fn obtain_value(&self, timestamp: usize) -> f64 {
        let chart = self.get_chart(timestamp);
        let mut value = self.get_balance();
        for symbol in self.get_symbols().iter() {
            let price = chart.get(symbol).as_ref().unwrap().adj_close;
            value += price * (self.get_share(symbol).unwrap() as f64);
        }
        value
    }

    pub fn run(&mut self, rolling_window: usize) -> BacktestReport {
        let mut timestamp = 1usize;
        let mut daily_return = vec![0f64; self.market_data.len()];
        let mut balance_history = vec![0f64; self.market_data.len()];
        let interst_rate = self.get_interest_rate();
        let daily_interest = (1f64 + interst_rate).powf(1f64 / 252f64) - 1f64;

        let mut total_value = self.obtain_value(timestamp);
        while timestamp <= self.market_data.len() {
            let idx = timestamp - 1;
            let chart_map = self.get_chart(timestamp).clone();
            let order_map = self
                .strategy
                .gen_order_map(timestamp, &chart_map, &self.portfolio);
            self.execute_order(&order_map, timestamp);

            // Interest
            let balance = self.get_balance();
            self.update_balance(balance * (1f64 + daily_interest));

            let new_value = self.obtain_value(timestamp);
            daily_return[idx] = (new_value - total_value) / total_value;
            total_value = new_value;
            balance_history[idx] = self.get_balance();
            timestamp += 1;
        }

        let cumulative_return = daily_return
            .iter()
            .scan(1f64, |state, x| {
                *state *= 1f64 + x;
                Some(*state)
            })
            .collect::<Vec<f64>>();

        let mut rolling_volatility = vec![0f64; daily_return.len()];
        for i in rolling_window..daily_return.len() {
            rolling_volatility[i] =
                daily_return[i - rolling_window..i].to_vec().sd() * 252f64.sqrt();
        }

        let mut rolling_sharpe_ratio = vec![0f64; daily_return.len()];
        let risk_free = self.get_risk_free();
        for i in rolling_window..daily_return.len() {
            let dr_roll = daily_return[i - rolling_window..i].to_vec();
            let rf = risk_free[i - rolling_window..i].to_vec();
            let excess_return = dr_roll.sub_v(&rf);
            rolling_sharpe_ratio[i] = excess_return.mean() / excess_return.sd() * 252f64.sqrt();
        }

        let mut drawdown = vec![0f64; daily_return.len()];
        let mut max = 0f64;
        for i in 0..daily_return.len() {
            if cumulative_return[i] > max {
                max = cumulative_return[i];
            } else {
                drawdown[i] = (max - cumulative_return[i]) / max;
            }
        }

        let cagr = cumulative_return
            .last()
            .unwrap()
            .powf(1f64 / (daily_return.len() as f64 / 252f64))
            - 1f64;
        let volatility = daily_return.sd() * (daily_return.len() as f64).sqrt();
        let sharpe_ratio = {
            let excess_return = daily_return.sub_v(risk_free);
            excess_return.mean() / excess_return.sd() * 252f64.sqrt()
        };
        let mdd = drawdown.max();

        BacktestReport {
            portfolio: self.portfolio.clone(),
            strategy: self.strategy.to_string(),
            market_data: self.market_data.clone(),
            daily_return,
            cumulative_return,
            rolling_volatility,
            rolling_sharpe_ratio,
            drawdown,
            cagr,
            volatility,
            sharpe_ratio,
            mdd,
            balance_history,
        }
    }
}

impl BacktestReport {
    pub fn get_daily_return(&self) -> &Vec<f64> {
        &self.daily_return
    }

    pub fn get_cumulative_return(&self) -> &Vec<f64> {
        &self.cumulative_return
    }

    pub fn get_rolling_volatility(&self) -> &Vec<f64> {
        &self.rolling_volatility
    }

    pub fn get_rolling_sharpe_ratio(&self) -> &Vec<f64> {
        &self.rolling_sharpe_ratio
    }

    pub fn get_drawdown(&self) -> &Vec<f64> {
        &self.drawdown
    }

    pub fn get_cagr(&self) -> f64 {
        self.cagr
    }

    pub fn get_volatility(&self) -> f64 {
        self.volatility
    }

    pub fn get_sharpe_ratio(&self) -> f64 {
        self.sharpe_ratio
    }

    pub fn get_mdd(&self) -> f64 {
        self.mdd
    }

    pub fn get_balance_history(&self) -> &Vec<f64> {
        &self.balance_history
    }

    pub fn to_dataframe(&self) -> DataFrame {
        let mut df = DataFrame::new(vec![]);
        df.push("date", Series::new(self.market_data.date.clone()));
        df.push("daily_return", Series::new(self.daily_return.clone()));
        df.push(
            "cumulative_return",
            Series::new(self.cumulative_return.clone()),
        );
        df.push(
            "rolling_volatility",
            Series::new(self.rolling_volatility.clone()),
        );
        df.push(
            "rolling_sharpe_ratio",
            Series::new(self.rolling_sharpe_ratio.clone()),
        );
        df.push("drawdown", Series::new(self.drawdown.clone()));
        df.push("balance_history", Series::new(self.balance_history.clone()));
        df
    }

    pub fn to_report(&self) -> DataFrame {
        let mut dg = DataFrame::new(vec![]);
        dg.push("Strategy", Series::new(vec![self.strategy.clone()]));
        dg.push("CAGR", Series::new(vec![self.cagr]));
        dg.push("Volatility", Series::new(vec![self.volatility]));
        dg.push("Sharpe", Series::new(vec![self.sharpe_ratio]));
        dg.push("MDD", Series::new(vec![self.mdd]));
        dg
    }
}

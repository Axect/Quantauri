use crate::{
    api::Chart,
    trade::{Order, Portfolio},
};
#[allow(unused_imports)]
use peroxide::fuga::*;
use std::collections::HashMap;

// ┌──────────────────────────────────────────────────────────┐
//  Rebalance Strategy
// └──────────────────────────────────────────────────────────┘
pub trait RebalanceStrategy {
    fn should_rebalance(
        &self,
        timestamp: usize,
        chart_map: &HashMap<String, Chart>,
        portfolio: &Portfolio,
    ) -> bool;
    fn reset(&mut self);
}

/// Periodic Rebalance
pub struct PeriodicRebalance {
    pub period: usize,
}

impl PeriodicRebalance {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl RebalanceStrategy for PeriodicRebalance {
    fn should_rebalance(
        &self,
        timestamp: usize,
        _chart_map: &HashMap<String, Chart>,
        _portfolio: &Portfolio,
    ) -> bool {
        timestamp % self.period == 0
    }
    fn reset(&mut self) {
        unimplemented!()
    }
}

/// Threshold Rebalance
///
/// - threshold : percentage of allocation deviation
pub struct ThresholdRebalance {
    pub threshold: f64,
    initial_weight: HashMap<String, f64>,
}

impl ThresholdRebalance {
    pub fn new(threshold: f64, initial_weight: &HashMap<String, f64>) -> Self {
        Self {
            threshold,
            initial_weight: initial_weight.clone(),
        }
    }

    pub fn get_initial_weight(&self, symbol: &str) -> Option<&f64> {
        self.initial_weight.get(symbol)
    }
}

impl RebalanceStrategy for ThresholdRebalance {
    fn should_rebalance(
        &self,
        _timestamp: usize,
        chart_map: &HashMap<String, Chart>,
        portfolio: &Portfolio,
    ) -> bool {
        let symbols = portfolio.get_symbols();
        let mut initial_weights = symbols
            .iter()
            .map(|s| *self.get_initial_weight(s).unwrap())
            .collect::<Vec<f64>>();
        let balance_weight = 1f64 - initial_weights.sum();
        initial_weights.push(balance_weight);

        let mut current_values = symbols
            .iter()
            .map(|s| {
                let chart = chart_map.get(s).unwrap();
                let price = chart.adj_close;
                let share = portfolio.get_share(s).unwrap();
                price * (share as f64)
            })
            .collect::<Vec<f64>>();
        let current_balance = portfolio.get_balance();
        current_values.push(current_balance);
        let current_weights = current_values.div_s(current_values.sum());

        let alloc_dev = zip_with(
            |c, i| (c - i).abs() * 100f64,
            &current_weights,
            &initial_weights,
        )
        .sum();
        alloc_dev > self.threshold
    }

    fn reset(&mut self) {
        unimplemented!()
    }
}

// ┌──────────────────────────────────────────────────────────┐
//  Strategy
// └──────────────────────────────────────────────────────────┘
pub trait Strategy {
    fn gen_order_map(
        &mut self,
        timestamp: usize,
        chart_map: &HashMap<String, Chart>,
        portfolio: &Portfolio,
    ) -> HashMap<String, Order>;
    fn to_string(&self) -> String;
}

/// Buy and Hold
pub struct BuyAndHold {
    pub weight: HashMap<String, f64>,
    rebalance_strategy: Box<dyn RebalanceStrategy>,
    bought: bool,
}

impl BuyAndHold {
    pub fn new(
        weight: HashMap<String, f64>,
        rebalance_strategy: Box<dyn RebalanceStrategy>,
    ) -> Self {
        Self {
            weight,
            rebalance_strategy,
            bought: false,
        }
    }

    pub fn get_weight(&self, symbol: &str) -> Option<&f64> {
        self.weight.get(symbol)
    }
}

impl Strategy for BuyAndHold {
    fn gen_order_map(
        &mut self,
        timestamp: usize,
        chart_map: &HashMap<String, Chart>,
        portfolio: &Portfolio,
    ) -> HashMap<String, Order> {
        let mut order_map = HashMap::new();
        let symbols = portfolio.get_symbols();
        for symbol in symbols.iter() {
            order_map.insert(symbol.to_string(), Order::new(symbol, 0));
        }

        if !self.bought {
            // Opening
            let current_balance = portfolio.balance;
            for symbol in symbols.iter() {
                let w = self.get_weight(symbol).unwrap();
                let current_price = chart_map.get(symbol).as_ref().unwrap().adj_close;
                let shares = (current_balance * w / current_price) as isize;
                order_map.insert(symbol.to_string(), Order::new(symbol, shares));
            }
            self.bought = true;
        } else if self
            .rebalance_strategy
            .should_rebalance(timestamp, chart_map, portfolio)
        {
            // Closing all positions
            for symbol in symbols.iter() {
                let current_share = portfolio.get_share(symbol).unwrap();
                order_map.insert(
                    symbol.to_string(),
                    Order::new(symbol, -(current_share as isize)),
                );
            }
            self.bought = false;
        }
        order_map
    }

    fn to_string(&self) -> String {
        "BnH".to_string()
    }
}

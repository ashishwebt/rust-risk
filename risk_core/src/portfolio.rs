use crate::black_scholes::{BsInputs, OptionType};
use serde::{Deserialize, Serialize};

/// A single option (or underlying) position held in the book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: u64,
    pub underlying_symbol: String,
    /// Live spot, refreshed from the market data feed.
    pub spot: f64,
    pub strike: f64,
    pub time_to_expiry: f64,
    pub rate: f64,
    pub dividend_yield: f64,
    /// Live/implied volatility used for pricing & Greeks.
    pub volatility: f64,
    pub option_type: OptionType,
    /// Signed number of contracts (negative = short).
    pub quantity: f64,
    /// Multiplier per contract (e.g. 100 shares per option contract).
    pub contract_multiplier: f64,
}

impl Position {
    pub fn bs_inputs(&self) -> BsInputs {
        BsInputs {
            spot: self.spot,
            strike: self.strike,
            time_to_expiry: self.time_to_expiry,
            rate: self.rate,
            dividend_yield: self.dividend_yield,
            volatility: self.volatility,
            option_type: self.option_type,
        }
    }

    pub fn notional_quantity(&self) -> f64 {
        self.quantity * self.contract_multiplier
    }
}

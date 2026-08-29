use crate::black_scholes::greeks;
use crate::portfolio::Position;

/// A single stress scenario expressed as relative/absolute shocks.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Scenario {
    pub name: String,
    /// Relative spot shock, e.g. -0.20 for a 20% crash.
    pub spot_shock_pct: f64,
    /// Absolute vol shock in vol points, e.g. 0.10 for +10 vol.
    pub vol_shock_abs: f64,
    /// Absolute rate shock, e.g. 0.01 for +100bps.
    pub rate_shock_abs: f64,
}

impl Scenario {
    pub fn market_crash() -> Self {
        Self {
            name: "Market Crash (-20%, vol +15, rates -50bp)".into(),
            spot_shock_pct: -0.20,
            vol_shock_abs: 0.15,
            rate_shock_abs: -0.005,
        }
    }
    pub fn vol_spike() -> Self {
        Self {
            name: "Vol Spike (vol +25)".into(),
            spot_shock_pct: 0.0,
            vol_shock_abs: 0.25,
            rate_shock_abs: 0.0,
        }
    }
    pub fn rally() -> Self {
        Self {
            name: "Rally (+15%, vol -5)".into(),
            spot_shock_pct: 0.15,
            vol_shock_abs: -0.05,
            rate_shock_abs: 0.0,
        }
    }
    pub fn rate_shock_up() -> Self {
        Self {
            name: "Rates +200bp".into(),
            spot_shock_pct: 0.0,
            vol_shock_abs: 0.0,
            rate_shock_abs: 0.02,
        }
    }

    pub fn default_set() -> Vec<Scenario> {
        vec![
            Scenario::market_crash(),
            Scenario::vol_spike(),
            Scenario::rally(),
            Scenario::rate_shock_up(),
        ]
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub base_value: f64,
    pub stressed_value: f64,
    pub pnl: f64,
}

/// Applies a scenario to every position and returns the aggregate P&L impact.
pub fn run_scenario(positions: &[Position], scenario: &Scenario) -> ScenarioResult {
    let mut base_value = 0.0;
    let mut stressed_value = 0.0;

    for pos in positions {
        let base_inputs = pos.bs_inputs();
        let base_price = base_inputs.price();
        base_value += base_price * pos.notional_quantity();

        let mut stressed_inputs = base_inputs;
        stressed_inputs.spot *= 1.0 + scenario.spot_shock_pct;
        stressed_inputs.volatility = (stressed_inputs.volatility + scenario.vol_shock_abs).max(0.001);
        stressed_inputs.rate += scenario.rate_shock_abs;
        let stressed_price = stressed_inputs.price();
        stressed_value += stressed_price * pos.notional_quantity();
    }

    ScenarioResult {
        scenario_name: scenario.name.clone(),
        base_value,
        stressed_value,
        pnl: stressed_value - base_value,
    }
}

pub fn run_all_scenarios(positions: &[Position], scenarios: &[Scenario]) -> Vec<ScenarioResult> {
    scenarios.iter().map(|s| run_scenario(positions, s)).collect()
}

/// Convenience: aggregate portfolio Greeks (sum of per-position Greeks * quantity).
pub fn aggregate_greeks(positions: &[Position]) -> crate::black_scholes::Greeks {
    let mut total = crate::black_scholes::Greeks::default();
    for pos in positions {
        let g = greeks(&pos.bs_inputs());
        total.delta += g.delta * pos.notional_quantity();
        total.gamma += g.gamma * pos.notional_quantity();
        total.vega += g.vega * pos.notional_quantity();
        total.theta += g.theta * pos.notional_quantity();
        total.rho += g.rho * pos.notional_quantity();
    }
    total
}

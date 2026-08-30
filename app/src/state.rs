use crate::db;
use data_feed::{
    FeedEvent, FeedManager, FeedStatus, MarketDataSource, MarketTick, MockPushAdapter,
    YahooPollAdapter,
};
use risk_core::{
    aggregate_greeks, historical_var, parametric_var, run_all_scenarios, Greeks, OptionType,
    Position, Scenario, ScenarioResult, VarConfig, VolSurface,
};
use rusqlite::Connection;
use std::collections::{HashMap, VecDeque};
use tracing::{debug, error, info, instrument, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceChoice {
    Simulated,
    Yahoo,
}

const PNL_HISTORY_LEN: usize = 250;

/// Maximum number of error entries kept in the in-app error log.
const ERROR_LOG_LEN: usize = 50;

/// A single entry in the rolling in-app error log.
#[derive(Debug, Clone)]
pub struct ErrorEntry {
    /// Human-readable timestamp (HH:MM:SS)
    pub timestamp: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Add-position form state
// ---------------------------------------------------------------------------

/// Which option type is selected in the add-position form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormOptionType {
    Call,
    Put,
}

impl From<FormOptionType> for OptionType {
    fn from(f: FormOptionType) -> Self {
        match f {
            FormOptionType::Call => OptionType::Call,
            FormOptionType::Put => OptionType::Put,
        }
    }
}

/// All mutable state for the "add position" form panel.
pub struct PositionForm {
    /// Whether the form panel is open.
    pub open: bool,
    pub symbol: String,
    pub strike: String,
    pub expiry: String,
    pub volatility: String,
    pub rate: String,
    pub dividend_yield: String,
    pub quantity: String,
    pub contract_multiplier: String,
    pub option_type: FormOptionType,
    /// Validation error shown inside the form.
    pub error: Option<String>,
}

impl Default for PositionForm {
    fn default() -> Self {
        Self {
            open: false,
            symbol: String::new(),
            strike: "100.0".into(),
            expiry: "0.25".into(),
            volatility: "0.25".into(),
            rate: "0.045".into(),
            dividend_yield: "0.0".into(),
            quantity: "1.0".into(),
            contract_multiplier: "100.0".into(),
            option_type: FormOptionType::Call,
            error: None,
        }
    }
}

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

pub struct AppState {
    pub db: Connection,

    pub feed_manager: FeedManager,
    pub feed_status: FeedStatus,
    pub source_choice: SourceChoice,

    pub positions: Vec<Position>,
    pub latest_ticks: HashMap<String, MarketTick>,
    pub pnl_history: HashMap<String, VecDeque<f64>>,

    pub vol_surface: VolSurface,
    pub var_config: VarConfig,
    pub scenarios: Vec<Scenario>,
    pub scenario_results: Vec<ScenarioResult>,

    pub portfolio_greeks: Greeks,

    /// Most-recent error string (kept for backward compat with any callers).
    pub last_error: Option<String>,

    /// Rolling log of the last `ERROR_LOG_LEN` errors shown in the UI panel.
    pub error_log: VecDeque<ErrorEntry>,

    /// State for the add-position form panel.
    pub position_form: PositionForm,
}

impl AppState {
    /// Build `AppState`, open / migrate the DB, load positions (seeding
    /// defaults if the table is empty), and start the data feed.
    pub fn new() -> Self {
        // --- DB setup ---
        let db = Connection::open("risk_dashboard.db")
            .expect("failed to open risk_dashboard.db");
        db::init_db(&db).expect("DB schema migration failed");

        let positions = db::seed_defaults_if_empty(&db).expect("failed to load/seed positions from DB");

        // --- Feed ---
        let symbols = unique_symbols(&positions);
        let mut feed_manager = FeedManager::new();
        let starting_prices = positions
            .iter()
            .map(|p| (p.underlying_symbol.clone(), p.spot))
            .collect::<HashMap<_, _>>();
        feed_manager.switch(
            Box::new(MockPushAdapter {
                starting_prices,
                volatility_per_tick: 0.0015,
            }),
            symbols,
        );

        // --- Vol surface ---
        let mut vol_surface = VolSurface::new(
            vec![90.0, 95.0, 100.0, 105.0, 110.0],
            vec![0.083, 0.25, 0.5, 1.0],
        );
        seed_vol_surface(&mut vol_surface);

        let scenarios = Scenario::default_set();
        let scenario_results = run_all_scenarios(&positions, &scenarios);
        let portfolio_greeks = aggregate_greeks(&positions);

        info!(
            position_count = positions.len(),
            source = "Simulated",
            "AppState initialized"
        );

        Self {
            db,
            feed_manager,
            feed_status: FeedStatus::Connecting,
            source_choice: SourceChoice::Simulated,
            positions,
            latest_ticks: HashMap::new(),
            pnl_history: HashMap::new(),
            vol_surface,
            var_config: VarConfig::default(),
            scenarios,
            scenario_results,
            portfolio_greeks,
            last_error: None,
            error_log: VecDeque::new(),
            position_form: PositionForm::default(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    // -----------------------------------------------------------------------
    // Internal helper: push an error into the rolling log and tracing.
    // -----------------------------------------------------------------------
    fn log_error(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        error!(message = %msg, "feed error");

        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.last_error = Some(msg.clone());
        self.error_log.push_back(ErrorEntry {
            timestamp,
            message: msg,
        });
        if self.error_log.len() > ERROR_LOG_LEN {
            self.error_log.pop_front();
        }
    }

    // -----------------------------------------------------------------------
    // Position management
    // -----------------------------------------------------------------------

    /// Add a position: persist to DB, push into in-memory list, re-subscribe feed.
    pub fn add_position(&mut self, pos: Position) {
        match db::save_position(&self.db, &pos) {
            Ok(saved) => {
                info!(id = saved.id, symbol = %saved.underlying_symbol, "position added");
                self.positions.push(saved);
                self.resubscribe_feed();
                self.recompute();
            }
            Err(e) => {
                self.log_error(format!("add_position DB error: {e}"));
            }
        }
    }

    /// Remove a position by id: delete from DB, remove from in-memory list, re-subscribe feed.
    pub fn remove_position(&mut self, id: u64) {
        match db::delete_position(&self.db, id) {
            Ok(_) => {
                self.positions.retain(|p| p.id != id);
                // Drop P&L history for symbols no longer in the book.
                let active: std::collections::HashSet<&str> = self
                    .positions
                    .iter()
                    .map(|p| p.underlying_symbol.as_str())
                    .collect();
                self.pnl_history.retain(|sym, _| active.contains(sym.as_str()));
                self.latest_ticks.retain(|sym, _| active.contains(sym.as_str()));
                info!(id, "position removed");
                self.resubscribe_feed();
                self.recompute();
            }
            Err(e) => {
                self.log_error(format!("remove_position DB error: {e}"));
            }
        }
    }

    /// Re-subscribe the active feed to the current symbol set.
    fn resubscribe_feed(&mut self) {
        let symbols = unique_symbols(&self.positions);
        let starting_prices = self
            .positions
            .iter()
            .map(|p| (p.underlying_symbol.clone(), p.spot))
            .collect::<HashMap<_, _>>();

        let adapter: Box<dyn MarketDataSource> = match self.source_choice {
            SourceChoice::Simulated => Box::new(MockPushAdapter {
                starting_prices,
                volatility_per_tick: 0.0015,
            }),
            SourceChoice::Yahoo => Box::new(YahooPollAdapter::default()),
        };
        self.feed_manager.switch(adapter, symbols);
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Drains every pending event from the feed channel.
    pub fn pump_feed(&mut self) {
        let rx = self.feed_manager.receiver();
        while let Ok(event) = rx.try_recv() {
            match event {
                FeedEvent::Tick(tick) => {
                    debug!(
                        symbol = %tick.symbol,
                        price = tick.price,
                        source = ?tick.source,
                        "market tick received"
                    );
                    self.apply_tick(tick);
                }
                FeedEvent::Status(status) => {
                    match &status {
                        FeedStatus::Connected => {
                            info!(source = %self.feed_manager.active_name(), "feed connected");
                        }
                        FeedStatus::Connecting => {
                            info!(source = %self.feed_manager.active_name(), "feed connecting");
                        }
                        FeedStatus::Disconnected(reason) => {
                            warn!(reason = %reason, "feed disconnected");
                        }
                        FeedStatus::Error(reason) => {
                            self.log_error(reason.clone());
                        }
                    }
                    self.feed_status = status;
                }
            }
        }
    }

    #[instrument(skip(self), fields(symbol = %tick.symbol, price = tick.price))]
    fn apply_tick(&mut self, tick: MarketTick) {
        let old_spot = self
            .latest_ticks
            .get(&tick.symbol)
            .map(|t| t.price)
            .unwrap_or(tick.price);

        for pos in self.positions.iter_mut() {
            if pos.underlying_symbol == tick.symbol {
                let pnl_delta = (tick.price - old_spot) * pos.notional_quantity();
                let history = self
                    .pnl_history
                    .entry(pos.underlying_symbol.clone())
                    .or_insert_with(VecDeque::new);
                history.push_back(pnl_delta);
                if history.len() > PNL_HISTORY_LEN {
                    history.pop_front();
                }
                pos.spot = tick.price;
            }
        }
        self.latest_ticks.insert(tick.symbol.clone(), tick);
        self.recompute();
    }

    #[instrument(skip(self))]
    pub fn recompute(&mut self) {
        self.portfolio_greeks = aggregate_greeks(&self.positions);
        self.scenario_results = run_all_scenarios(&self.positions, &self.scenarios);
        debug!("risk metrics recomputed");
    }

    #[instrument(skip(self), fields(choice = ?choice))]
    pub fn switch_source(&mut self, choice: SourceChoice) {
        info!(from = ?self.source_choice, to = ?choice, "switching data source");
        self.source_choice = choice;
        self.resubscribe_feed();
    }

    /// Combined historical-simulation VaR across all positions' P&L histories.
    pub fn portfolio_historical_var(&self) -> f64 {
        let n = self
            .pnl_history
            .values()
            .map(|v| v.len())
            .max()
            .unwrap_or(0);
        if n == 0 {
            return 0.0;
        }
        let mut combined = vec![0.0; n];
        for history in self.pnl_history.values() {
            for (i, v) in history.iter().rev().enumerate() {
                if i < n {
                    combined[n - 1 - i] += v;
                }
            }
        }
        historical_var(&combined, &self.var_config)
    }

    pub fn portfolio_parametric_var(&self) -> f64 {
        let portfolio_value: f64 = self
            .positions
            .iter()
            .map(|p| p.bs_inputs().price() * p.notional_quantity())
            .sum();
        let avg_vol = if self.positions.is_empty() {
            0.0
        } else {
            self.positions.iter().map(|p| p.volatility).sum::<f64>() / self.positions.len() as f64
        };
        let daily_vol = avg_vol / (252f64).sqrt();
        parametric_var(portfolio_value.abs(), daily_vol, &self.var_config)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn unique_symbols(positions: &[Position]) -> Vec<String> {
    positions
        .iter()
        .map(|p| p.underlying_symbol.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

fn seed_vol_surface(surface: &mut VolSurface) {
    let base = 0.20;
    for (ei, expiry) in surface.expiries.clone().iter().enumerate() {
        for (ki, strike) in surface.strikes.clone().iter().enumerate() {
            let moneyness = (strike - 100.0) / 100.0;
            let smile = 0.15 * moneyness * moneyness;
            let term_decay = 0.05 / (expiry + 0.1);
            surface.set(ei, ki, base + smile + term_decay);
        }
    }
}

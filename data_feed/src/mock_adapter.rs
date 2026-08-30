use crate::event::{DataSourceKind, FeedEvent, FeedStatus, MarketTick};
use crate::source::MarketDataSource;
use crossbeam_channel::Sender;
use rand::RngExt;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, info};

/// Simulates a push-style feed (irregular arrival times, like a websocket)
/// via a random walk. Useful for demoing/dev without network access, and
/// for proving the UI genuinely doesn't care whether ticks are pushed or
/// pulled: swapping this for `YahooPollAdapter` requires zero UI changes.
pub struct MockPushAdapter {
    pub starting_prices: HashMap<String, f64>,
    pub volatility_per_tick: f64,
}

impl Default for MockPushAdapter {
    fn default() -> Self {
        Self {
            starting_prices: HashMap::new(),
            volatility_per_tick: 0.002,
        }
    }
}

impl MarketDataSource for MockPushAdapter {
    fn name(&self) -> &str {
        "Simulated (push)"
    }

    fn start(&self, symbols: Vec<String>, tx: Sender<FeedEvent>) {
        let mut prices: HashMap<String, f64> = symbols
            .iter()
            .map(|s| {
                let start = self.starting_prices.get(s).copied().unwrap_or(100.0);
                (s.clone(), start)
            })
            .collect();
        let vol = self.volatility_per_tick;

        info!(
            symbols = ?symbols,
            volatility_per_tick = vol,
            "MockPushAdapter starting"
        );

        std::thread::spawn(move || {
            let _ = tx.send(FeedEvent::Status(FeedStatus::Connecting));
            let mut rng = rand::rng();
            let _ = tx.send(FeedEvent::Status(FeedStatus::Connected));
            loop {
                for symbol in symbols.iter() {
                    let price = prices.entry(symbol.clone()).or_insert(100.0);
                    let shock: f64 = rng.random_range(-1.0..1.0) * vol;
                    *price = (*price * (1.0 + shock)).max(0.01);

                    debug!(
                        symbol = %symbol,
                        price = *price,
                        shock = shock,
                        "mock tick generated"
                    );

                    let tick = MarketTick {
                        symbol: symbol.clone(),
                        price: *price,
                        bid: Some(*price * 0.9995),
                        ask: Some(*price * 1.0005),
                        volume: Some(rng.random_range(100..10_000)),
                        timestamp: chrono::Utc::now().timestamp(),
                        source: DataSourceKind::MockPush,
                    };
                    if tx.send(FeedEvent::Tick(tick)).is_err() {
                        info!("MockPushAdapter: receiver dropped, shutting down");
                        return;
                    }
                }
                // Irregular arrival to mimic a real push feed.
                let sleep_ms = rand::rng().random_range(150..900);
                std::thread::sleep(Duration::from_millis(sleep_ms));
            }
        });
    }
}

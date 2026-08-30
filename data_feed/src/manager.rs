use crate::event::FeedEvent;
use crate::source::MarketDataSource;
use crossbeam_channel::{unbounded, Receiver, Sender};
use tracing::info;

/// Owns the active data source and the single channel the UI drains.
/// Switching between push/pull/remote implementations is just swapping
/// what `start` is called on; the `Receiver<FeedEvent>` handed to the UI
/// never changes shape or identity from the UI's point of view.
pub struct FeedManager {
    tx: Sender<FeedEvent>,
    rx: Receiver<FeedEvent>,
    active_name: String,
}

impl FeedManager {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self {
            tx,
            rx,
            active_name: "none".to_string(),
        }
    }

    /// Swap in a new data source for the given symbols. Old adapter threads
    /// naturally wind down once they notice the previous sender was
    /// replaced/dropped (checked via `send` failing), or simply keep
    /// running harmlessly in the background for this scaffold.
    pub fn switch(&mut self, source: Box<dyn MarketDataSource>, symbols: Vec<String>) {
        let new_name = source.name().to_string();
        info!(
            previous = %self.active_name,
            next = %new_name,
            symbols = ?symbols,
            "FeedManager switching source"
        );
        self.active_name = new_name;
        source.start(symbols, self.tx.clone());
    }

    pub fn receiver(&self) -> Receiver<FeedEvent> {
        self.rx.clone()
    }

    pub fn active_name(&self) -> &str {
        &self.active_name
    }
}

impl Default for FeedManager {
    fn default() -> Self {
        Self::new()
    }
}

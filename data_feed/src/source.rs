use crate::event::FeedEvent;
use crossbeam_channel::Sender;

/// The single abstraction the UI depends on. Whether an implementation
/// polls a REST endpoint, subscribes to a WebSocket, or generates mock
/// data, it only needs to push `FeedEvent`s into `tx`. The UI's consume
/// loop (see `app`) is identical no matter which implementation runs.
///
/// This is intentionally a trait object friendly, `Send`-bound interface so
/// a `Box<dyn MarketDataSource>` can be swapped at runtime (e.g. a settings
/// toggle between "Live (Yahoo)" and "Simulated").
pub trait MarketDataSource: Send {
    /// Human-readable name shown in the UI (e.g. "Yahoo Finance (poll)").
    fn name(&self) -> &str;

    /// Start streaming. Implementations should spawn their own task/thread
    /// and return immediately; they keep pushing `FeedEvent`s into `tx`
    /// until the channel is dropped or `symbols` changes (a new call to
    /// `start` with an updated list is the simplest way to change
    /// subscriptions for this scaffold).
    fn start(&self, symbols: Vec<String>, tx: Sender<FeedEvent>);
}

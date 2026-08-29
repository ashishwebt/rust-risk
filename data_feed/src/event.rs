use serde::{Deserialize, Serialize};

/// A single normalized market tick. This is the ONLY shape the UI and
/// business logic ever see, regardless of whether it originated from a
/// pull-based poll (REST/yahoo_finance_api), a push-based stream
/// (WebSocket/gRPC), or a mock generator. Every adapter's job is purely
/// translation into this shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketTick {
    pub symbol: String,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub volume: Option<u64>,
    /// Unix timestamp (seconds) of the tick.
    pub timestamp: i64,
    pub source: DataSourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataSourceKind {
    YahooPull,
    MockPush,
    /// Reserved for a future server relay (e.g. gRPC/WebSocket) once the
    /// client is split out; the UI layer requires no changes to consume it.
    RemoteServer,
}

/// Connection lifecycle events, also normalized across transports so the UI
/// can show one consistent "connected / reconnecting / error" indicator no
/// matter which adapter is active.
#[derive(Debug, Clone, PartialEq)]
pub enum FeedStatus {
    Connecting,
    Connected,
    Disconnected(String),
    Error(String),
}

/// Everything the UI drains from the channel: either a price tick or a
/// status change. One unified stream, one unified consumer loop.
#[derive(Debug, Clone)]
pub enum FeedEvent {
    Tick(MarketTick),
    Status(FeedStatus),
}

use crate::event::{DataSourceKind, FeedEvent, FeedStatus, MarketTick};
use crate::source::MarketDataSource;
use crossbeam_channel::Sender;
use std::time::Duration;
use yahoo_finance_api as yahoo;

/// Pulls quotes from Yahoo Finance on a fixed interval and translates each
/// response into `MarketTick`s. This is a "pull" transport, but the UI
/// never knows that: it only ever sees `FeedEvent`s on the shared channel,
/// exactly like it would from a push-based websocket adapter.
pub struct YahooPollAdapter {
    pub poll_interval: Duration,
}

impl Default for YahooPollAdapter {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
        }
    }
}

impl MarketDataSource for YahooPollAdapter {
    fn name(&self) -> &str {
        "Yahoo Finance (poll)"
    }

    fn start(&self, symbols: Vec<String>, tx: Sender<FeedEvent>) {
        let interval = self.poll_interval;
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(FeedEvent::Status(FeedStatus::Error(format!(
                        "failed to start tokio runtime: {e}"
                    ))));
                    return;
                }
            };
            rt.block_on(async move {
                let provider = yahoo::YahooConnector::new();
                let _ = tx.send(FeedEvent::Status(FeedStatus::Connecting));
                let mut first = true;
                loop {
                    if tx.is_full() {
                        // no-op branch kept for clarity; crossbeam channels here are unbounded
                    }
                    if tx.send(FeedEvent::Status(FeedStatus::Connected)).is_err() {
                        return; // receiver dropped, stop polling
                    }
                    for symbol in &symbols {
                        match provider.get_latest_quotes(symbol, "1d").await {
                            Ok(response) => {
                                if let Ok(quote) = response.last_quote() {
                                    let tick = MarketTick {
                                        symbol: symbol.clone(),
                                        price: quote.close,
                                        bid: None,
                                        ask: None,
                                        volume: Some(quote.volume),
                                        timestamp: quote.timestamp as i64,
                                        source: DataSourceKind::YahooPull,
                                    };
                                    if tx.send(FeedEvent::Tick(tick)).is_err() {
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(FeedEvent::Status(FeedStatus::Error(format!(
                                    "{symbol}: {e}"
                                ))));
                            }
                        }
                    }
                    if first {
                        first = false;
                    }
                    tokio::time::sleep(interval).await;
                }
            });
        });
    }
}

use crate::event::{DataSourceKind, FeedEvent, FeedStatus, MarketTick};
use crate::source::MarketDataSource;
use crossbeam_channel::Sender;
use std::time::Duration;
use tracing::{error, info, warn};
use yahoo_finance_api as yahoo;

/// Polls Yahoo Finance on a fixed interval and translates each response into
/// `MarketTick`s on the shared channel. Uses `yahoo_finance_api` v4 which
/// sets a proper User-Agent header, avoiding the 429 rate-limit errors seen
/// with v2.x.
pub struct YahooPollAdapter {
    pub poll_interval: Duration,
}

impl Default for YahooPollAdapter {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(15),
        }
    }
}

impl MarketDataSource for YahooPollAdapter {
    fn name(&self) -> &str {
        "Yahoo Finance (poll)"
    }

    fn start(&self, symbols: Vec<String>, tx: Sender<FeedEvent>) {
        let interval = self.poll_interval;
        info!(
            symbols = ?symbols,
            interval_secs = interval.as_secs(),
            "YahooPollAdapter starting"
        );

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    error!(error = %e, "failed to create tokio runtime");
                    let _ = tx.send(FeedEvent::Status(FeedStatus::Error(format!(
                        "tokio runtime error: {e}"
                    ))));
                    return;
                }
            };

            rt.block_on(async move {
                let provider = match yahoo::YahooConnector::new() {
                    Ok(p) => p,
                    Err(e) => {
                        error!(error = %e, "failed to build Yahoo connector");
                        let _ = tx.send(FeedEvent::Status(FeedStatus::Error(format!(
                            "failed to build Yahoo connector: {e}"
                        ))));
                        return;
                    }
                };

                let _ = tx.send(FeedEvent::Status(FeedStatus::Connecting));

                loop {
                    if tx.send(FeedEvent::Status(FeedStatus::Connected)).is_err() {
                        info!("YahooPollAdapter: receiver dropped, shutting down");
                        return;
                    }

                    for symbol in &symbols {
                        match provider.get_latest_quotes(symbol, "1d").await {
                            Ok(response) => match response.last_quote() {
                                Ok(quote) => {
                                    info!(
                                        symbol = %symbol,
                                        price = quote.close,
                                        volume = quote.volume,
                                        "Yahoo tick received"
                                    );
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
                                        info!("YahooPollAdapter: receiver dropped mid-send, shutting down");
                                        return;
                                    }
                                }
                                Err(e) => {
                                    warn!(symbol = %symbol, error = %e, "no quote data in Yahoo response");
                                    let _ = tx.send(FeedEvent::Status(FeedStatus::Error(
                                        format!("{symbol}: no quote data ({e})"),
                                    )));
                                }
                            },
                            Err(e) => {
                                error!(symbol = %symbol, error = %e, "Yahoo Finance request failed");
                                let _ = tx.send(FeedEvent::Status(FeedStatus::Error(format!(
                                    "{symbol}: {e}"
                                ))));
                            }
                        }
                    }

                    tokio::time::sleep(interval).await;
                }
            });
        });
    }
}

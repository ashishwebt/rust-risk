//! Provider enum — which data source feeds a position.
//!
//! Kept in the `app` crate so `risk_core` stays pure maths.
//! The DB stores a comma-separated list e.g. `"Simulated,Yahoo"`.

/// Which market-data provider(s) should feed a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
    Simulated,
    Yahoo,
}

impl Provider {
    pub fn label(self) -> &'static str {
        match self {
            Provider::Simulated => "Simulated",
            Provider::Yahoo => "Yahoo",
        }
    }

    /// All variants in display order.
    pub fn all() -> &'static [Provider] {
        &[Provider::Simulated, Provider::Yahoo]
    }
}

// ---------------------------------------------------------------------------
// Comma-separated DB serialization
// ---------------------------------------------------------------------------

/// Serialize a provider list to a DB string, e.g. `"Simulated,Yahoo"`.
/// An empty list serializes to `""` (treated as "all providers" on load
/// for safety, but the form prevents submitting with none selected).
pub fn providers_to_str(providers: &[Provider]) -> String {
    providers
        .iter()
        .map(|p| p.label())
        .collect::<Vec<_>>()
        .join(",")
}

/// Parse a DB string back to a provider list.
/// Unknown tokens are silently skipped so old DB rows stay valid.
pub fn providers_from_str(s: &str) -> Vec<Provider> {
    if s.is_empty() {
        // Fallback: treat missing/empty as all providers.
        return Provider::all().to_vec();
    }
    s.split(',')
        .filter_map(|tok| match tok.trim() {
            "Simulated" => Some(Provider::Simulated),
            "Yahoo" => Some(Provider::Yahoo),
            _ => None,
        })
        .collect()
}

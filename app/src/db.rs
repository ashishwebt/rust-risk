//! SQLite persistence for positions.
//!
//! The database file lives next to the binary at `risk_dashboard.db`.
//! All schema migrations are applied on startup via `init_db`.

use rusqlite::{params, Connection, Result};
use risk_core::{OptionType, Position};
use tracing::{info, instrument};

/// Open (or create) the database and ensure the schema is up to date.
#[instrument]
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS positions (
            id                 INTEGER PRIMARY KEY,
            underlying_symbol  TEXT    NOT NULL,
            spot               REAL    NOT NULL,
            strike             REAL    NOT NULL,
            time_to_expiry     REAL    NOT NULL,
            rate               REAL    NOT NULL,
            dividend_yield     REAL    NOT NULL,
            volatility         REAL    NOT NULL,
            option_type        TEXT    NOT NULL CHECK(option_type IN ('Call','Put')),
            quantity           REAL    NOT NULL,
            contract_multiplier REAL   NOT NULL
        );",
    )?;
    info!("DB schema ready");
    Ok(())
}

/// Load all positions ordered by id.
pub fn load_positions(conn: &Connection) -> Result<Vec<Position>> {
    let mut stmt = conn.prepare(
        "SELECT id, underlying_symbol, spot, strike, time_to_expiry,
                rate, dividend_yield, volatility, option_type,
                quantity, contract_multiplier
         FROM positions ORDER BY id",
    )?;

    let rows = stmt.query_map([], |row| {
        let opt_str: String = row.get(8)?;
        let option_type = if opt_str == "Call" {
            OptionType::Call
        } else {
            OptionType::Put
        };
        Ok(Position {
            id: row.get::<_, i64>(0)? as u64,
            underlying_symbol: row.get(1)?,
            spot: row.get(2)?,
            strike: row.get(3)?,
            time_to_expiry: row.get(4)?,
            rate: row.get(5)?,
            dividend_yield: row.get(6)?,
            volatility: row.get(7)?,
            option_type,
            quantity: row.get(9)?,
            contract_multiplier: row.get(10)?,
        })
    })?;

    rows.collect()
}

/// Persist a new position. The `id` on the returned position is the
/// SQLite-assigned ROWID so callers should use the returned value.
pub fn save_position(conn: &Connection, pos: &Position) -> Result<Position> {
    let opt_str = match pos.option_type {
        OptionType::Call => "Call",
        OptionType::Put => "Put",
    };

    conn.execute(
        "INSERT INTO positions
         (underlying_symbol, spot, strike, time_to_expiry, rate,
          dividend_yield, volatility, option_type, quantity, contract_multiplier)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
        params![
            pos.underlying_symbol,
            pos.spot,
            pos.strike,
            pos.time_to_expiry,
            pos.rate,
            pos.dividend_yield,
            pos.volatility,
            opt_str,
            pos.quantity,
            pos.contract_multiplier,
        ],
    )?;

    let id = conn.last_insert_rowid() as u64;
    info!(id, symbol = %pos.underlying_symbol, "position saved to DB");
    Ok(Position { id, ..pos.clone() })
}

/// Delete a position by id. Returns `true` if a row was actually deleted.
pub fn delete_position(conn: &Connection, id: u64) -> Result<bool> {
    let affected = conn.execute("DELETE FROM positions WHERE id = ?1", params![id as i64])?;
    info!(id, deleted = affected > 0, "delete_position");
    Ok(affected > 0)
}

/// If the positions table is empty, insert the default starting book.
/// Returns the positions that are now in the DB (whether freshly seeded or pre-existing).
pub fn seed_defaults_if_empty(conn: &Connection) -> Result<Vec<Position>> {
    let mut positions = load_positions(conn)?;
    if positions.is_empty() {
        info!("DB empty — seeding default positions");
        for p in default_positions() {
            let saved = save_position(conn, &p)?;
            positions.push(saved);
        }
    }
    Ok(positions)
}

// ---------------------------------------------------------------------------
// Default starting book — single source of truth
// ---------------------------------------------------------------------------

fn default_positions() -> Vec<Position> {
    use risk_core::OptionType;
    vec![
        Position {
            id: 0,
            underlying_symbol: "AAPL".into(),
            spot: 225.0,
            strike: 230.0,
            time_to_expiry: 0.25,
            rate: 0.045,
            dividend_yield: 0.005,
            volatility: 0.28,
            option_type: OptionType::Call,
            quantity: 10.0,
            contract_multiplier: 100.0,
        },
        Position {
            id: 0,
            underlying_symbol: "AAPL".into(),
            spot: 225.0,
            strike: 210.0,
            time_to_expiry: 0.5,
            rate: 0.045,
            dividend_yield: 0.005,
            volatility: 0.30,
            option_type: OptionType::Put,
            quantity: -5.0,
            contract_multiplier: 100.0,
        },
        Position {
            id: 0,
            underlying_symbol: "MSFT".into(),
            spot: 420.0,
            strike: 430.0,
            time_to_expiry: 0.17,
            rate: 0.045,
            dividend_yield: 0.007,
            volatility: 0.24,
            option_type: OptionType::Call,
            quantity: 8.0,
            contract_multiplier: 100.0,
        },
        Position {
            id: 0,
            underlying_symbol: "SPY".into(),
            spot: 560.0,
            strike: 540.0,
            time_to_expiry: 1.0,
            rate: 0.045,
            dividend_yield: 0.013,
            volatility: 0.16,
            option_type: OptionType::Put,
            quantity: 20.0,
            contract_multiplier: 100.0,
        },
    ]
}

//! SQLite persistence for positions.
//!
//! The database file lives next to the binary at `risk_dashboard.db`.
//! All schema migrations are applied on startup via `init_db`.

use crate::provider::{providers_from_str, providers_to_str, Provider};
use risk_core::{OptionType, Position};
use rusqlite::{params, Connection, Result};
use tracing::{info, instrument};

/// Open (or create) the database and ensure the schema is up to date.
///
/// Migration strategy: create the table if missing, then use
/// `ALTER TABLE … ADD COLUMN IF NOT EXISTS` for each new column so
/// existing DBs without the `providers` column are upgraded automatically.
#[instrument]
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS positions (
            id                  INTEGER PRIMARY KEY,
            underlying_symbol   TEXT    NOT NULL,
            spot                REAL    NOT NULL,
            strike              REAL    NOT NULL,
            time_to_expiry      REAL    NOT NULL,
            rate                REAL    NOT NULL,
            dividend_yield      REAL    NOT NULL,
            volatility          REAL    NOT NULL,
            option_type         TEXT    NOT NULL CHECK(option_type IN ('Call','Put')),
            quantity            REAL    NOT NULL,
            contract_multiplier REAL    NOT NULL
        );",
    )?;

    // Additive migration: add providers column if it doesn't exist yet.
    // SQLite doesn't support IF NOT EXISTS on ALTER TABLE, so we catch
    // the error when the column already exists.
    let _ = conn.execute_batch(
        "ALTER TABLE positions ADD COLUMN providers TEXT NOT NULL DEFAULT 'Simulated';",
    );

    info!("DB schema ready");
    Ok(())
}

// ---------------------------------------------------------------------------
// CRUD
// ---------------------------------------------------------------------------

/// Load all positions ordered by id, returning both the `Position` and
/// its provider list as a pair.
pub fn load_positions(conn: &Connection) -> Result<Vec<(Position, Vec<Provider>)>> {
    let mut stmt = conn.prepare(
        "SELECT id, underlying_symbol, spot, strike, time_to_expiry,
                rate, dividend_yield, volatility, option_type,
                quantity, contract_multiplier, providers
         FROM positions ORDER BY id",
    )?;

    let rows = stmt.query_map([], |row| {
        let opt_str: String = row.get(8)?;
        let option_type = if opt_str == "Call" {
            OptionType::Call
        } else {
            OptionType::Put
        };
        let providers_str: String = row.get(11)?;
        Ok((
            Position {
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
            },
            providers_from_str(&providers_str),
        ))
    })?;

    rows.collect()
}

/// Persist a new position with its providers.
/// Returns the position with the DB-assigned id.
pub fn save_position(
    conn: &Connection,
    pos: &Position,
    providers: &[Provider],
) -> Result<Position> {
    let opt_str = match pos.option_type {
        OptionType::Call => "Call",
        OptionType::Put => "Put",
    };
    let providers_str = providers_to_str(providers);

    conn.execute(
        "INSERT INTO positions
         (underlying_symbol, spot, strike, time_to_expiry, rate,
          dividend_yield, volatility, option_type, quantity, contract_multiplier, providers)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
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
            providers_str,
        ],
    )?;

    let id = conn.last_insert_rowid() as u64;
    info!(id, symbol = %pos.underlying_symbol, providers = %providers_str, "position saved to DB");
    Ok(Position { id, ..pos.clone() })
}

/// Delete a position by id. Returns `true` if a row was actually deleted.
pub fn delete_position(conn: &Connection, id: u64) -> Result<bool> {
    let affected = conn.execute("DELETE FROM positions WHERE id = ?1", params![id as i64])?;
    info!(id, deleted = affected > 0, "delete_position");
    Ok(affected > 0)
}

/// If the positions table is empty, insert the default starting book.
/// Returns `(positions, providers_map)` for all rows now in the DB.
pub fn seed_defaults_if_empty(conn: &Connection) -> Result<Vec<(Position, Vec<Provider>)>> {
    let mut rows = load_positions(conn)?;
    if rows.is_empty() {
        info!("DB empty — seeding default positions");
        for (p, providers) in default_positions() {
            let saved = save_position(conn, &p, &providers)?;
            rows.push((saved, providers));
        }
    }
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Default starting book — single source of truth
// ---------------------------------------------------------------------------

fn default_positions() -> Vec<(Position, Vec<Provider>)> {
    vec![
        (
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
            vec![Provider::Simulated, Provider::Yahoo],
        ),
        (
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
            vec![Provider::Simulated, Provider::Yahoo],
        ),
        (
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
            vec![Provider::Yahoo],
        ),
        (
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
            vec![Provider::Simulated],
        ),
    ]
}

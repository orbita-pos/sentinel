use crate::error::AppError;
use rusqlite::Connection;

use super::schema;

struct Migration {
    version: i32,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "foundation",
        sql: schema::MIGRATION_001_FOUNDATION,
    },
    Migration {
        version: 2,
        name: "matches",
        sql: schema::MIGRATION_002_MATCHES,
    },
    Migration {
        version: 3,
        name: "static_data",
        sql: schema::MIGRATION_003_STATIC_DATA,
    },
    Migration {
        version: 4,
        name: "mastery",
        sql: schema::MIGRATION_004_MASTERY,
    },
    Migration {
        version: 5,
        name: "patterns",
        sql: schema::MIGRATION_005_PATTERNS,
    },
];

pub fn run_all(conn: &Connection) -> Result<(), AppError> {
    // Create migrations table if it doesn't exist
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version     INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            applied_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for migration in MIGRATIONS {
        if migration.version > current_version {
            tracing::info!(
                "Running migration {}: {}",
                migration.version,
                migration.name
            );
            conn.execute_batch(migration.sql)?;
            conn.execute(
                "INSERT INTO _migrations (version, name) VALUES (?1, ?2)",
                rusqlite::params![migration.version, migration.name],
            )?;
        }
    }

    tracing::info!(
        "Database at version {} ({} migrations applied)",
        MIGRATIONS.last().map_or(0, |m| m.version),
        MIGRATIONS.len()
    );

    Ok(())
}

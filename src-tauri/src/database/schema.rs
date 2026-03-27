/// Migration 001: Foundation tables
pub const MIGRATION_001_FOUNDATION: &str = "
CREATE TABLE IF NOT EXISTS summoners (
    puuid           TEXT PRIMARY KEY,
    game_name       TEXT NOT NULL,
    tag_line        TEXT NOT NULL,
    summoner_id     TEXT,
    account_id      TEXT,
    profile_icon_id INTEGER,
    summoner_level  INTEGER,
    region          TEXT NOT NULL,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS app_state (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);
";

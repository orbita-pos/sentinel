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

/// Migration 002: Match history and timeline storage
pub const MIGRATION_002_MATCHES: &str = "
CREATE TABLE IF NOT EXISTS matches (
    match_id            TEXT PRIMARY KEY,
    game_creation       INTEGER NOT NULL,
    game_duration       INTEGER NOT NULL,
    game_mode           TEXT NOT NULL,
    queue_id            INTEGER NOT NULL,
    game_version        TEXT,
    raw_json            TEXT NOT NULL,
    fetched_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS match_participants (
    match_id            TEXT NOT NULL REFERENCES matches(match_id),
    participant_id      INTEGER NOT NULL,
    puuid               TEXT NOT NULL,
    champion_id         INTEGER NOT NULL,
    champion_name       TEXT NOT NULL,
    team_id             INTEGER NOT NULL,
    team_position       TEXT,
    kills               INTEGER NOT NULL DEFAULT 0,
    deaths              INTEGER NOT NULL DEFAULT 0,
    assists             INTEGER NOT NULL DEFAULT 0,
    total_minions_killed INTEGER NOT NULL DEFAULT 0,
    gold_earned         INTEGER NOT NULL DEFAULT 0,
    total_damage_dealt  INTEGER NOT NULL DEFAULT 0,
    vision_score        INTEGER NOT NULL DEFAULT 0,
    win                 INTEGER NOT NULL,
    PRIMARY KEY (match_id, participant_id)
);

CREATE INDEX IF NOT EXISTS idx_match_participants_puuid ON match_participants(puuid);
CREATE INDEX IF NOT EXISTS idx_matches_game_creation ON matches(game_creation DESC);

CREATE TABLE IF NOT EXISTS match_timelines (
    match_id            TEXT PRIMARY KEY REFERENCES matches(match_id),
    raw_json            TEXT NOT NULL,
    fetched_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS match_events (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id            TEXT NOT NULL REFERENCES matches(match_id),
    event_type          TEXT NOT NULL,
    timestamp_ms        INTEGER NOT NULL,
    participant_id      INTEGER,
    data_json           TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_match_events_match ON match_events(match_id);
CREATE INDEX IF NOT EXISTS idx_match_events_type ON match_events(event_type);
";

/// Migration 003: Static data and player profiles
pub const MIGRATION_003_STATIC_DATA: &str = "
CREATE TABLE IF NOT EXISTS champions (
    champion_id         INTEGER PRIMARY KEY,
    champion_key        TEXT NOT NULL,
    name                TEXT NOT NULL,
    title               TEXT,
    tags                TEXT,
    patch_version       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS items (
    item_id             INTEGER PRIMARY KEY,
    name                TEXT NOT NULL,
    description         TEXT,
    gold_total          INTEGER,
    gold_base           INTEGER,
    tags                TEXT,
    from_items          TEXT,
    into_items          TEXT,
    patch_version       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS player_profiles (
    puuid               TEXT PRIMARY KEY,
    game_name           TEXT,
    tag_line            TEXT,
    tier                TEXT,
    rank                TEXT,
    league_points       INTEGER,
    wins                INTEGER,
    losses              INTEGER,
    updated_at          TEXT NOT NULL DEFAULT (datetime('now'))
);
";

/// Migration 004: Champion mastery cache
pub const MIGRATION_004_MASTERY: &str = "
CREATE TABLE IF NOT EXISTS champion_mastery (
    puuid               TEXT NOT NULL,
    champion_id         INTEGER NOT NULL,
    mastery_level       INTEGER NOT NULL DEFAULT 0,
    mastery_points      INTEGER NOT NULL DEFAULT 0,
    updated_at          TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (puuid, champion_id)
);
";

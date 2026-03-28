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

/// Migration 005: Pattern engine, post-game analysis, improvement tracking
pub const MIGRATION_005_PATTERNS: &str = "
CREATE TABLE IF NOT EXISTS game_features (
    match_id            TEXT NOT NULL,
    puuid               TEXT NOT NULL,
    champion_id         INTEGER NOT NULL,
    role                TEXT,
    win                 INTEGER NOT NULL,
    game_duration_min   REAL NOT NULL,
    cs_at_10            INTEGER,
    cs_at_15            INTEGER,
    gold_diff_at_10     INTEGER,
    gold_diff_at_15     INTEGER,
    gold_diff_at_20     INTEGER,
    deaths_before_15    INTEGER,
    deaths_after_25     INTEGER,
    vision_score_per_min REAL,
    kill_participation  REAL,
    had_early_lead      INTEGER,
    threw_lead          INTEGER,
    features_json       TEXT NOT NULL,
    computed_at         TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (match_id, puuid)
);

CREATE TABLE IF NOT EXISTS detected_patterns (
    id                  TEXT PRIMARY KEY,
    puuid               TEXT NOT NULL,
    category            TEXT NOT NULL,
    description         TEXT NOT NULL,
    confidence          REAL NOT NULL,
    sample_size         INTEGER NOT NULL,
    impact_wr_change    REAL,
    impact_games_pct    REAL,
    trend               TEXT NOT NULL DEFAULT 'Stable',
    evidence_json       TEXT NOT NULL DEFAULT '[]',
    first_detected      TEXT NOT NULL DEFAULT (datetime('now')),
    last_updated        TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_patterns_puuid ON detected_patterns(puuid);
CREATE INDEX IF NOT EXISTS idx_game_features_puuid ON game_features(puuid);

CREATE TABLE IF NOT EXISTS post_game_analyses (
    match_id            TEXT PRIMARY KEY,
    puuid               TEXT NOT NULL,
    analysis_json       TEXT NOT NULL,
    created_at          TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS improvement_goals (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    puuid               TEXT NOT NULL,
    name                TEXT NOT NULL,
    description         TEXT,
    metric_key          TEXT NOT NULL,
    target_value        REAL,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    linked_pattern_id   TEXT,
    active              INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS improvement_snapshots (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    puuid               TEXT NOT NULL,
    metric_key          TEXT NOT NULL,
    value               REAL NOT NULL,
    match_count         INTEGER NOT NULL,
    snapshot_date       TEXT NOT NULL,
    UNIQUE(puuid, metric_key, snapshot_date)
);

CREATE INDEX IF NOT EXISTS idx_snapshots_puuid ON improvement_snapshots(puuid, metric_key, snapshot_date);
";

/// Migration 006: Live game timeline capture (no API key needed)
pub const MIGRATION_006_LIVE_CAPTURE: &str = "
CREATE TABLE IF NOT EXISTS live_snapshots (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id          TEXT NOT NULL,
    game_time           REAL NOT NULL,
    player_name         TEXT NOT NULL,
    champion            TEXT NOT NULL,
    team                TEXT NOT NULL,
    level               INTEGER NOT NULL,
    kills               INTEGER NOT NULL,
    deaths              INTEGER NOT NULL,
    assists             INTEGER NOT NULL,
    cs                  INTEGER NOT NULL,
    ward_score          REAL NOT NULL DEFAULT 0,
    item_gold           INTEGER NOT NULL DEFAULT 0,
    is_local            INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_live_snap_session ON live_snapshots(session_id, game_time);

CREATE TABLE IF NOT EXISTS live_game_events (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id          TEXT NOT NULL,
    game_time           REAL NOT NULL,
    event_name          TEXT NOT NULL,
    description         TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_live_events_session ON live_game_events(session_id);

CREATE TABLE IF NOT EXISTS live_sessions (
    session_id          TEXT PRIMARY KEY,
    match_id            TEXT,
    puuid               TEXT NOT NULL,
    local_champion      TEXT NOT NULL DEFAULT '',
    game_mode           TEXT NOT NULL DEFAULT '',
    game_duration       REAL NOT NULL DEFAULT 0,
    win                 INTEGER,
    started_at          TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at            TEXT
);
";

use std::path::PathBuf;

use super::types::LockfileData;

/// Known default League installation paths on Windows
const DEFAULT_LOCKFILE_PATHS: &[&str] = &[
    "C:/Riot Games/League of Legends/lockfile",
    "D:/Riot Games/League of Legends/lockfile",
    "E:/Riot Games/League of Legends/lockfile",
    "C:/Program Files/Riot Games/League of Legends/lockfile",
    "C:/Program Files (x86)/Riot Games/League of Legends/lockfile",
    "D:/Program Files/Riot Games/League of Legends/lockfile",
    "C:/Games/Riot Games/League of Legends/lockfile",
    "D:/Games/Riot Games/League of Legends/lockfile",
];

/// Try to read and parse the lockfile from known paths
pub fn try_read_lockfile() -> Option<LockfileData> {
    for path_str in DEFAULT_LOCKFILE_PATHS {
        let path = PathBuf::from(path_str);
        if let Some(data) = try_read_lockfile_at(&path) {
            return Some(data);
        }
    }

    // Also check Riot Client install path from registry-like locations
    // Try common Riot install folder patterns
    if let Some(home) = std::env::var("LOCALAPPDATA").ok() {
        let riot_path = PathBuf::from(&home)
            .join("Riot Games")
            .join("League of Legends")
            .join("lockfile");
        if let Some(data) = try_read_lockfile_at(&riot_path) {
            return Some(data);
        }
    }

    None
}

/// Try to read a lockfile at a specific path.
/// Retries once if the file is locked by the League process.
fn try_read_lockfile_at(path: &PathBuf) -> Option<LockfileData> {
    if !path.exists() {
        return None;
    }

    // First attempt
    if let Ok(content) = std::fs::read_to_string(path) {
        if !content.trim().is_empty() {
            return LockfileData::parse(&content);
        }
    }

    // Retry after short delay (file may be locked by League during startup)
    std::thread::sleep(std::time::Duration::from_millis(100));
    let content = std::fs::read_to_string(path).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    LockfileData::parse(&content)
}

/// Detect LCU credentials from lockfile only.
/// No process queries (wmic/tasklist) -- those trigger Vanguard anti-cheat.
pub fn detect() -> Option<LockfileData> {
    try_read_lockfile()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_paths_exist() {
        // Just verify the constant is populated
        assert!(!DEFAULT_LOCKFILE_PATHS.is_empty());
    }
}

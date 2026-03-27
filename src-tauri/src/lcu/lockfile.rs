use std::path::PathBuf;
use std::process::Command;

use super::types::LockfileData;

/// Known default League installation paths on Windows
const DEFAULT_LOCKFILE_PATHS: &[&str] = &[
    "C:/Riot Games/League of Legends/lockfile",
    "D:/Riot Games/League of Legends/lockfile",
    "C:/Program Files/Riot Games/League of Legends/lockfile",
    "C:/Program Files (x86)/Riot Games/League of Legends/lockfile",
];

/// Try to read and parse the lockfile from known paths
pub fn try_read_lockfile() -> Option<LockfileData> {
    for path_str in DEFAULT_LOCKFILE_PATHS {
        let path = PathBuf::from(path_str);
        if let Some(data) = try_read_lockfile_at(&path) {
            return Some(data);
        }
    }
    None
}

/// Try to read a lockfile at a specific path
fn try_read_lockfile_at(path: &PathBuf) -> Option<LockfileData> {
    // The lockfile is often locked by the League process.
    // We need to read it despite the lock.
    // On Windows, opening with share mode allows reading locked files.
    // std::fs::read_to_string handles this on most systems.
    let content = std::fs::read_to_string(path).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    LockfileData::parse(&content)
}

/// Fallback: extract port and auth token from LeagueClientUx.exe command line
pub fn try_from_process() -> Option<LockfileData> {
    // Use wmic to get the command line of LeagueClientUx.exe
    let output = Command::new("wmic")
        .args([
            "process",
            "where",
            "name='LeagueClientUx.exe'",
            "get",
            "CommandLine",
            "/format:list",
        ])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_process_commandline(&stdout)
}

/// Parse the wmic output to extract --app-port and --remoting-auth-token
fn parse_process_commandline(output: &str) -> Option<LockfileData> {
    let line = output.lines().find(|l| l.contains("--app-port"))?;

    let port = extract_arg(line, "--app-port=")?;
    let token = extract_arg(line, "--remoting-auth-token=")?;
    let pid_str = extract_arg(line, "--app-pid=").unwrap_or_else(|| "0".to_string());

    Some(LockfileData {
        pid: pid_str.parse().unwrap_or(0),
        port: port.parse().ok()?,
        password: token,
        protocol: "https".to_string(),
    })
}

/// Extract a value from a command line argument like --key=value
fn extract_arg(line: &str, prefix: &str) -> Option<String> {
    let start = line.find(prefix)? + prefix.len();
    let rest = &line[start..];
    // Value ends at next space or quote or end of string
    let end = rest
        .find(|c: char| c == ' ' || c == '"')
        .unwrap_or(rest.len());
    let value = rest[..end].trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// Detect LCU credentials: try lockfile first, then process fallback
pub fn detect() -> Option<LockfileData> {
    try_read_lockfile().or_else(try_from_process)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_arg() {
        let line = r#""C:\Riot Games\LeagueClientUx.exe" --app-port=8443 --remoting-auth-token=abc123 --app-pid=9876"#;
        assert_eq!(extract_arg(line, "--app-port="), Some("8443".into()));
        assert_eq!(
            extract_arg(line, "--remoting-auth-token="),
            Some("abc123".into())
        );
        assert_eq!(extract_arg(line, "--app-pid="), Some("9876".into()));
        assert_eq!(extract_arg(line, "--nonexistent="), None);
    }

    #[test]
    fn test_parse_process_commandline() {
        let output = r#"
CommandLine="C:\Riot Games\LeagueClientUx.exe" --app-port=8443 --remoting-auth-token=tokenXYZ --app-pid=1234
"#;
        let data = parse_process_commandline(output).unwrap();
        assert_eq!(data.port, 8443);
        assert_eq!(data.password, "tokenXYZ");
        assert_eq!(data.pid, 1234);
    }
}

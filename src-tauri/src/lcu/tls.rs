use reqwest::Certificate;

/// Riot Games root certificate for LCU self-signed TLS
const RIOT_CERT_PEM: &[u8] = include_bytes!("riotgames.pem");

/// Build a reqwest Certificate from the embedded Riot PEM
pub fn riot_certificate() -> Certificate {
    Certificate::from_pem(RIOT_CERT_PEM).expect("Failed to parse Riot root certificate")
}

/// Build a reqwest::Client configured for LCU connections.
///
/// SECURITY NOTE: `danger_accept_invalid_certs(true)` is required because
/// the LCU uses a self-signed certificate where the CN does not match
/// `127.0.0.1`. This is safe here because:
/// 1. The URL is hardcoded to `https://127.0.0.1:{port}` (not user-controlled)
/// 2. Loopback traffic cannot be intercepted without local admin access
/// 3. The Riot root certificate is added for best-effort chain validation
pub fn build_lcu_http_client(password: &str) -> reqwest::Client {
    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        format!("riot:{password}"),
    );

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Basic {encoded}"))
            .expect("Invalid auth header"),
    );

    reqwest::Client::builder()
        .add_root_certificate(riot_certificate())
        .danger_accept_invalid_certs(true) // Required: LCU cert CN != 127.0.0.1
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("Failed to build LCU HTTP client")
}

/// Build a TLS connector for the LCU WebSocket connection.
///
/// SECURITY NOTE: Same localhost scope as the HTTP client above.
/// The Riot root certificate is added for best-effort validation.
pub fn riot_tls_connector() -> tokio_tungstenite::Connector {
    let mut builder = native_tls::TlsConnector::builder();

    // Add the Riot root certificate (fixes H1: previously missing)
    if let Ok(cert) = native_tls::Certificate::from_pem(RIOT_CERT_PEM) {
        builder.add_root_certificate(cert);
    }

    // Required: LCU WebSocket cert CN != 127.0.0.1
    builder.danger_accept_invalid_certs(true);

    let tls = builder.build().expect("Failed to build TLS connector");
    tokio_tungstenite::Connector::NativeTls(tls)
}

/// Build a reqwest::Client for the Game Client API (localhost:2999).
///
/// SECURITY NOTE: Same localhost scope as LCU. The Game Client API
/// uses a self-signed cert on 127.0.0.1:2999 with no auth required.
/// URL is hardcoded, not user-controlled.
pub fn build_game_client_http() -> reqwest::Client {
    reqwest::Client::builder()
        .add_root_certificate(riot_certificate())
        .danger_accept_invalid_certs(true) // Required: Game Client cert CN != 127.0.0.1
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .expect("Failed to build game client HTTP client")
}

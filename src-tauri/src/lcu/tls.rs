use reqwest::Certificate;

/// Riot Games root certificate for LCU self-signed TLS
const RIOT_CERT_PEM: &[u8] = include_bytes!("riotgames.pem");

/// Build a reqwest Certificate from the embedded Riot PEM
pub fn riot_certificate() -> Certificate {
    Certificate::from_pem(RIOT_CERT_PEM).expect("Failed to parse Riot root certificate")
}

/// Build a reqwest::Client configured for LCU connections
/// (accepts Riot self-signed cert, basic auth)
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
        .danger_accept_invalid_certs(true) // Cert is for "127.0.0.1" which won't match CN
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("Failed to build LCU HTTP client")
}

/// Build a rustls ClientConfig that trusts the Riot root cert
/// Used for the WebSocket TLS connection
pub fn riot_tls_connector() -> tokio_tungstenite::Connector {
    // For the WebSocket we also need to accept invalid certs
    // because the cert CN won't match 127.0.0.1
    let tls = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build TLS connector");
    tokio_tungstenite::Connector::NativeTls(tls)
}

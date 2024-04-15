// src/config.rs

use std::env;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Config {
    pub tls_proxy_host: String,
    pub tls_proxy_port: u16,
    pub tls_proxy_cpo_cert: String,
    pub tls_proxy_pri_key: String,
    pub tls_proxy_ca_cert: String,
    pub ccs_client_ip: String,
    pub ccs_client_port: String,
}

impl Config {
    /// Creates a new configuration, loading values from environment variables.
    /// Returns an Arc wrapped Config object for safe sharing across threads.
    pub fn new() -> Result<Arc<Self>, String> {
        let tls_proxy_host = env::var("TLS_PROXY_HOST").map_err(|e| e.to_string())?;
        let tls_proxy_port = env::var("TLS_PROXY_PORT")
            .map_err(|e| e.to_string())?
            .parse::<u16>()
            .map_err(|e| e.to_string())?;
        let tls_proxy_cpo_cert = env::var("TLS_PROXY_CPO_CERT").map_err(|e| e.to_string())?;
        let tls_proxy_pri_key = env::var("TLS_PROXY_PRIV_KEY").map_err(|e| e.to_string())?;
        let tls_proxy_ca_cert = env::var("TLS_PROXY_CA_CERT").map_err(|e| e.to_string())?;
        let ccs_client_ip = env::var("CCS_CLIENT_IP").map_err(|e| e.to_string())?;
        let ccs_client_port = env::var("CCS_CLIENT_PORT").map_err(|e| e.to_string())?;

        Ok(Arc::new(Config {
            tls_proxy_host,
            tls_proxy_port,
            tls_proxy_cpo_cert,
            tls_proxy_pri_key,
            tls_proxy_ca_cert,
            ccs_client_ip,
            ccs_client_port,
        }))
    }
}

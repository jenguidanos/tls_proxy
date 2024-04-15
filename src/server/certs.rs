use rustls::internal::pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use rustls::{AllowAnyAuthenticatedClient, ServerConfig};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader};

#[derive(Debug)]
pub struct PemfileError;

impl fmt::Display for PemfileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to read PEM file")
    }
}

impl Error for PemfileError {}
unsafe impl Send for PemfileError {}
unsafe impl Sync for PemfileError {}

pub fn load_tls_config(
    cert_path: &str,
    key_path: &str,
    ca_path: &str,
) -> Result<ServerConfig, Box<dyn Error + Send + Sync>> {
    let cert_file = &mut BufReader::new(File::open(cert_path)?);
    let key_file = &mut BufReader::new(File::open(key_path)?);
    let ca_file = &mut BufReader::new(File::open(ca_path)?);

    let cert_chain = certs(cert_file).map_err(|_| PemfileError)?;
    let mut keys = rsa_private_keys(key_file).map_err(|_| PemfileError)?;

    if keys.is_empty() {
        keys = pkcs8_private_keys(key_file).map_err(|_| PemfileError)?;
    }

    let mut config = ServerConfig::new(AllowAnyAuthenticatedClient::new(load_ca_certs(ca_file)?));
    config.set_single_cert(cert_chain, keys.remove(0))?;

    Ok(config)
}

fn load_ca_certs(reader: &mut BufReader<File>) -> Result<rustls::RootCertStore, io::Error> {
    let mut root_store = rustls::RootCertStore::empty();
    if root_store.add_pem_file(reader).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid CA certificate",
        ));
    }
    Ok(root_store)
}

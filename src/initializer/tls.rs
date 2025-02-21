use std::sync::Arc;

use axum_server::tls_rustls::RustlsConfig;
use rustls::{
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
    version::{TLS12, TLS13},
    ServerConfig,
};

pub fn build_tls_config() -> anyhow::Result<RustlsConfig> {
    let cert_path = dotenvy::var("CERT_PATH").unwrap();
    let key_path = dotenvy::var("KEY_PATH").unwrap();
    let certs = CertificateDer::pem_file_iter(cert_path)
        .unwrap()
        .map(|cert| cert.unwrap())
        .collect();
    let private_key = PrivateKeyDer::from_pem_file(key_path).unwrap();
    let server_config = ServerConfig::builder_with_protocol_versions(&[&TLS12, &TLS13])
        .with_no_client_auth()
        .with_single_cert(certs, private_key)?;

    Ok(RustlsConfig::from_config(Arc::new(server_config)))
}

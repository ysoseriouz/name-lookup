use std::{future::Future, net::SocketAddr, sync::Arc};

use axum::{
    handler::HandlerWithoutStateExt,
    http::{uri::Authority, StatusCode, Uri},
    response::Redirect,
    BoxError,
};
use axum_extra::extract::Host;
use axum_server::tls_rustls::RustlsConfig;
use rustls::{
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
    version::{TLS12, TLS13},
    ServerConfig,
};

#[derive(Clone, Copy)]
pub struct Ports {
    pub http: u16,
    pub https: u16,
}

pub async fn redirect_http_to_https<F>(ports: Ports, signal: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    fn make_https(host: &str, uri: Uri, https_port: u16) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let authority: Authority = host.parse()?;
        let bare_host = match authority.port() {
            Some(port_struct) => authority
                .as_str()
                .strip_suffix(port_struct.as_str())
                .unwrap()
                .strip_suffix(':')
                .unwrap(),
            None => authority.as_str(),
        };
        parts.authority = Some(format!("{bare_host}:{https_port}").parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(&host, uri, ports.https) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, redirect.into_make_service())
        .with_graceful_shutdown(signal)
        .await
        .unwrap();
}

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

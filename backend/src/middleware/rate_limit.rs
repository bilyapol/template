use axum::http::Request;
use std::net::IpAddr;
use tower_governor::GovernorError;
use tower_governor::key_extractor::{KeyExtractor, PeerIpKeyExtractor, SmartIpKeyExtractor};

/// Rate-limit key: the client IP.
///
/// By default uses the TCP peer address, which clients cannot fake. With
/// `trust_proxy` it reads X-Forwarded-For / X-Real-IP instead — required behind
/// a reverse proxy (otherwise every user shares the proxy's IP), but only safe
/// when that proxy is the sole way to reach the server.
#[derive(Debug, Clone, Copy)]
pub struct ClientIpKeyExtractor {
    pub trust_proxy: bool,
}

impl KeyExtractor for ClientIpKeyExtractor {
    type Key = IpAddr;

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        if self.trust_proxy {
            SmartIpKeyExtractor.extract(req)
        } else {
            PeerIpKeyExtractor.extract(req)
        }
    }
}

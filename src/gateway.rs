use crate::config::server::Source;
use async_trait::async_trait;
use log::{error, info};
use pingora::http::ResponseHeader;
use pingora::listeners::ALPN;
use pingora::prelude::{HttpPeer, ProxyHttp, Session};
use pingora::protocols::l4::socket::SocketAddr;
use pingora::upstreams::peer::{PeerOptions, Scheme};
use pingora::{Error, HTTPStatus};
use std::collections::{BTreeMap, HashMap};
use std::net::ToSocketAddrs;
use std::time::Duration;

pub struct Gateway {
    port: u16,
    routes: HashMap<String, (String, Source)>,
}

impl Gateway {
    pub fn new(port: u16, routes: HashMap<String, (String, Source)>) -> Self {
        Self { port, routes }
    }

    fn peer(self: &Self, source: &(String, Source)) -> Box<HttpPeer> {
        let addr = (source.1.ip.as_str(), source.1.port);

        let domain = match &source.1.host {
            Some(domain) => domain.clone(),
            None => source.1.ip.clone(),
        };

        let mut addrs_iter = addr.to_socket_addrs().unwrap();
        let addr = addrs_iter.next().unwrap();
        Box::new(HttpPeer {
            _address: SocketAddr::Inet(addr),
            scheme: Scheme::from_tls_bool(source.1.ssl),
            sni: domain,
            proxy: None,
            client_cert_key: None,
            options: PeerOptions {
                bind_to: None,
                connection_timeout: Some(Duration::new(3, 0)),
                total_connection_timeout: None,
                read_timeout: None,
                idle_timeout: None,
                write_timeout: None,
                verify_cert: source.1.ssl,
                verify_hostname: source.1.ssl,
                alternative_cn: None,
                alpn: ALPN::H1,
                ca: None,
                tcp_keepalive: None,
                no_header_eos: false,
                h2_ping_interval: None,
                max_h2_streams: 1,
                extra_proxy_headers: BTreeMap::new(),
                curves: None,
                second_keyshare: true, // default true and noop when not using PQ curves
                tracer: None,
            },
        })
    }
}

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let sni = match session.downstream_session.get_header("Host") {
            None => "",
            Some(host) => host.to_str().unwrap(),
        };

        let source: &(String, Source) = match self.routes.get(sni) {
            None => match self.routes.get("") {
                None => {
                    error!("[{}] No route match for {}", self.port, sni);
                    return Err(Box::new(*Error::new(HTTPStatus(502))));
                }
                Some(source) => source,
            },
            Some(source) => source,
        };

        info!(
            "[{}.{}]: {} {} {:?}",
            self.port,
            source.0,
            session.downstream_session.req_header().method,
            session.downstream_session.req_header().uri,
            session.downstream_session.req_header().headers
        );

        match &source.1.host {
            Some(domain) => {
                session
                    .req_header_mut()
                    .insert_header("Host", domain)
                    .expect("Failed");
            }
            None => {}
        };

        let peer = self.peer(source);

        info!("{:?}", session.req_header());
        info!("{peer:?}");
        Ok(peer)
    }

    async fn request_filter(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<bool> {
        Ok(false)
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // replace any existing header
        upstream_response
            .insert_header("Server", "Pingpong")
            .unwrap();

        // doesnt support h3
        upstream_response.remove_header("alt-svc");
        Ok(())
    }
}

use crate::config::Source;
use async_trait::async_trait;
use log::{error, info};
use pingora::http::ResponseHeader;
use pingora::prelude::{HttpPeer, ProxyHttp, Session};
use pingora::{Error, HTTPStatus};
use std::collections::HashMap;
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

        let mut peer = HttpPeer::new(addr, source.1.ssl, domain);

        peer.options.connection_timeout = Some(Duration::new(3, 0));

        Box::new(peer)
    }
}

pub struct GatewayCTX {
    pub source_match: Option<String>,
}

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = GatewayCTX;
    fn new_ctx(&self) -> Self::CTX {
        GatewayCTX { source_match: None }
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let sni = match session.downstream_session.get_header("Host") {
            None => "",
            Some(host) => host.to_str().unwrap(),
        };

        let source: &(String, Source) = match self.routes.get(sni) {
            None => match self.routes.get("") {
                None => {
                    error!("[{}] No route match for {}", self.port, sni);
                    return Err(Error::new(HTTPStatus(502)));
                }
                Some(source) => source,
            },
            Some(source) => source,
        };

        ctx.source_match = Some(String::from(&source.0));

        info!(
            "[{}.{}]: {} {} {:?}",
            self.port,
            source.0,
            session.downstream_session.req_header().method,
            session.downstream_session.req_header().uri,
            session.downstream_session.req_header().headers
        );

        if let Some(domain) = &source.1.host {
            session.req_header_mut().insert_header("Host", domain)?;
        };

        if let Some(heads) = &source.1.headers_request {
            for head in heads {
                session
                    .req_header_mut()
                    .insert_header(String::from(head.0), head.1)?;
            }
        }

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
        ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        // replace any existing header
        upstream_response.insert_header("Server", "Pingpong")?;

        // doesnt support h3
        upstream_response.remove_header("alt-svc");

        if let Some(source_match) = &ctx.source_match {
            let source: &(String, Source) = match self.routes.get(source_match) {
                None => self.routes.get("").unwrap(),
                Some(source) => source,
            };

            if let Some(heads) = &source.1.headers_response {
                for head in heads {
                    upstream_response.insert_header(String::from(head.0), head.1)?;
                }
            }
        }

        Ok(())
    }
}

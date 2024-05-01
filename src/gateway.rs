use crate::config::{Location, RewriteFlag, Source};
use async_trait::async_trait;
use http::Uri;
use log::{error, info};
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::prelude::{HttpPeer, ProxyHttp, Session};
use pingora::{Error, HTTPStatus};
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::time::Duration;
use urlencoding::encode;

pub struct Gateway {
    port: u16,
    routes: HashMap<String, HashMap<String, Source>>,
    check_status: bool,
}

impl Gateway {
    pub fn new(
        port: u16,
        routes: HashMap<String, HashMap<String, Source>>,
        check_status: bool,
    ) -> Self {
        Self {
            port,
            routes,
            check_status,
        }
    }

    fn peer(&self, source: (&String, &Source)) -> Box<HttpPeer> {
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
    pub sni: Option<String>,
    pub source: Option<String>,
}

fn match_route(uri: &str, source: &Source) -> bool {
    for location in &source.location {
        if match location {
            Location::Start(l) => uri.starts_with(l),
            Location::Equal(l) => uri.eq(l),
            Location::Regex(re) => re.is_match(uri),
        } {
            return true;
        }
    }
    false
}

fn find_route_with_start<'a>(
    sni: &'a str,
    uri: &str,
    routes: &'a HashMap<String, HashMap<String, Source>>,
    depth: usize,
    ctx: &mut GatewayCTX,
    starts_from: (&'a String, &'a Source),
) -> pingora::Result<((&'a String, &'a Source), String)> {
    let mut uri = String::from(encode(uri));
    let mut result: Option<pingora::Result<((&'a String, &'a Source), String)>> = None;
    if let Some(rewrites) = &starts_from.1.rewrite {
        for rewrite in rewrites {
            if result.is_some() {
                break;
            }
            if rewrite.0.is_match(&uri) {
                uri = rewrite.0.replace_all(&uri, &rewrite.1).to_string();
                if let RewriteFlag::Last = rewrite.2 {
                    result = Some(find_route(sni, &uri, routes, depth + 1, ctx));
                }
            }
        }
    }
    match result {
        None => Ok(((starts_from.0, starts_from.1), String::from(&uri))),
        Some(result) => result,
    }
}

fn find_route<'a>(
    sni: &'a str,
    uri: &str,
    routes: &'a HashMap<String, HashMap<String, Source>>,
    depth: usize,
    ctx: &mut GatewayCTX,
) -> pingora::Result<((&'a String, &'a Source), String)> {
    if depth >= 10 {
        Err(Error::new(HTTPStatus(502)))?;
    }
    let mut source: Option<(&String, &Source)> = None;
    if let Some(sni_sources) = routes.get(sni) {
        ctx.sni = Some(String::from(sni));
        for s in sni_sources {
            if match_route(uri, s.1) {
                source = Some(s);
            }
        }
    };
    if source.is_none() {
        if let Some(sni_sources) = routes.get("") {
            ctx.sni = Some(String::from(""));
            for s in sni_sources {
                if match_route(uri, s.1) {
                    source = Some(s);
                }
            }
        }
    }
    match source {
        None => Err(Error::new(HTTPStatus(502)))?,
        Some(s) => find_route_with_start(sni, uri, routes, depth, ctx, s),
    }
}

fn check_status(source: &Source) -> bool {
    if let Some(lb) = &source.load_balancer {
        if lb.select(b"", 256).is_some() {
            return true;
        }
    }
    false
}

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = GatewayCTX;
    fn new_ctx(&self) -> Self::CTX {
        GatewayCTX {
            sni: None,
            source: None,
        }
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let sni = match session.downstream_session.get_header("Host") {
            None => String::from(""),
            Some(host) => String::from(host.to_str().unwrap()),
        };
        let header: &mut RequestHeader = session.req_header_mut();

        let uri = &header.uri.to_string();
        let uri_raw = String::from(uri);

        let (source, uri) = {
            if self.check_status {
                let mut re: ((&String, &Source), String) =
                    find_route(&sni, uri, &self.routes, 0, ctx)?;

                for _ in 0..10 {
                    if check_status(re.0 .1) {
                        break;
                    } else {
                        for fallback in &re.0 .1.fallback {
                            re = find_route_with_start(
                                &sni,
                                &re.1,
                                &self.routes,
                                0,
                                ctx,
                                (
                                    fallback,
                                    match self.routes.get(&sni).unwrap().get(fallback) {
                                        Some(source) => source,
                                        None => Err(Error::new(HTTPStatus(502)))?,
                                    },
                                ),
                            )?;
                            if check_status(re.0 .1) {
                                break;
                            }
                        }
                    }
                }
                re
            } else {
                find_route(&sni, uri, &self.routes, 0, ctx)?
            }
        };

        ctx.source = Some(String::from(source.0));

        info!(
            "[{}.{}]: {} {} {:?}",
            self.port, source.0, header.method, uri_raw, header.headers
        );

        header.set_uri(uri.parse::<Uri>().or({
            error!(
                "[{}.{}]: Failed to parse rewritten uri: {}",
                self.port, source.0, &uri
            );
            Err(Error::new(HTTPStatus(502)))
        })?);

        if let Some(domain) = &source.1.host {
            header.insert_header("Host", domain)?;
        };

        if let Some(heads) = &source.1.headers_request {
            for head in heads {
                header.insert_header(String::from(head.0), head.1)?;
            }
        }

        let peer = self.peer(source);

        info!("{:?}", session.req_header_mut());
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

        if let Some(sni) = &ctx.sni {
            if let Some(s) = &ctx.source {
                let source: &Source = self.routes.get(sni).unwrap().get(s).unwrap();

                if let Some(heads) = &source.headers_response {
                    for head in heads {
                        upstream_response.insert_header(String::from(head.0), head.1)?;
                    }
                }
            }
        }

        Ok(())
    }
}

use crate::config::{Proxy, Source};
use crate::util::file_err::{make_page50x, PAGE404};
use crate::util::mime::get_mime_type;
use crate::util::path;
use crate::util::route::*;
use crate::util::url::encode_ignore_slash;
use async_trait::async_trait;
use http::{header, StatusCode, Uri};
use log::{error, info};
use pingora::http::{RequestHeader, ResponseHeader};
use pingora::prelude::{HttpPeer, ProxyHttp, Session};
use pingora::{Error, HTTPStatus};
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::time::Duration;
use urlencoding::decode;

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

    fn peer(&self, source: &Proxy) -> Box<HttpPeer> {
        let addr = (source.ip.as_str(), source.port);

        let domain = match &source.host {
            Some(domain) => domain.clone(),
            None => source.ip.clone(),
        };

        let mut addrs_iter = addr.to_socket_addrs().unwrap();
        let addr = addrs_iter.next().unwrap();

        let mut peer = HttpPeer::new(addr, source.ssl, domain);

        peer.options.connection_timeout = Some(Duration::new(3, 0));

        Box::new(peer)
    }
}

pub struct GatewayCTX {
    pub sni: Option<String>,
    pub source: Option<String>,
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
        let header: &mut RequestHeader = session.req_header_mut();
        if let Some(sni) = &ctx.sni {
            if let Some(s) = &ctx.source {
                let source: &Source = self.routes.get(sni).unwrap().get(s).unwrap();
                let source = match source {
                    Source::Proxy(proxy) => proxy,
                    Source::Static(_) => Err(Error::new(HTTPStatus(502)))?,
                };

                if let Some(domain) = &source.host {
                    header.insert_header("Host", domain)?;
                };

                if let Some(heads) = &source.headers_request {
                    for head in heads {
                        header.insert_header(String::from(head.0), head.1)?;
                    }
                }

                let peer = self.peer(source);

                return Ok(peer);
            }
        };

        Err(Error::new(HTTPStatus(502)))
    }

    async fn request_filter(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> pingora::Result<bool> {
        let sni = match session.downstream_session.get_header("Host") {
            None => String::from(""),
            Some(host) => String::from(host.to_str().unwrap()),
        };
        let header: &mut RequestHeader = session.req_header_mut();

        let uri = encode_ignore_slash(&header.uri.to_string()).into_owned();
        let uri_raw = String::from(&header.uri.to_string());

        let (source, uri) = {
            if self.check_status {
                let mut re: ((&String, &Source), String) =
                    find_route(&sni, &uri, &self.routes, 0, ctx).inspect_err(|_| {
                        error!("[{}]: Failed to find route {}", self.port, &uri_raw);
                    })?;

                for _ in 0..10 {
                    if check_status(re.0 .1, re.1.as_str()) {
                        break;
                    } else {
                        for fallback in re.0 .1.fallback_as_ref() {
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
                                        None => {
                                            error!(
                                                "[{}]: Failed to find fallback source {}",
                                                self.port, fallback
                                            );
                                            return make_page50x(session, StatusCode::BAD_GATEWAY)
                                                .await;
                                        }
                                    },
                                ),
                            )?;
                            if check_status(re.0 .1, re.1.as_str()) {
                                break;
                            }
                        }
                    }
                }
                re
            } else {
                find_route(&sni, &uri, &self.routes, 0, ctx)?
            }
        };

        ctx.source = Some(String::from(source.0));

        info!(
            "[{}.{}]: {} \"{}\" \"{}\" \"{}\"",
            self.port,
            source.0,
            header.method,
            sni,
            uri_raw,
            match header.headers.get("User-Agent") {
                None => "",
                Some(ua) => {
                    ua.to_str().unwrap()
                }
            }
        );

        header.set_uri(
            match match decode(&uri) {
                Ok(uri) => uri,
                Err(e) => {
                    error!(
                        "[{}.{}]: Failed to parse rewritten uri: {}, {}",
                        self.port,
                        source.0,
                        &uri,
                        e.to_string()
                    );
                    return make_page50x(session, StatusCode::BAD_GATEWAY).await;
                }
            }
            .into_owned()
            .parse::<Uri>()
            {
                Ok(uri) => uri,
                Err(e) => {
                    error!(
                        "[{}.{}]: Failed to parse rewritten uri: {}, {}",
                        self.port,
                        source.0,
                        &uri,
                        e.to_string()
                    );
                    return make_page50x(session, StatusCode::BAD_GATEWAY).await;
                }
            },
        );

        match source.1 {
            Source::Proxy(_) => {}
            Source::Static(source) => {
                let mut status = StatusCode::OK;
                let mut file_path = path::resolve_uri(&source.root, uri.as_str());
                file_path = file_path.split('?').collect::<Vec<&str>>()[0].to_string();
                if file_path.ends_with('/') {
                    file_path = format!("{}index.html", file_path)
                }
                let file = match std::fs::read(&file_path) {
                    Ok(file) => file,
                    Err(_) => {
                        error!("File not exist: {}", &file_path);
                        info!("File not exist: {}", &file_path);
                        file_path = path::resolve_uri(&source.root, "/404.html");
                        status = StatusCode::NOT_FOUND;
                        std::fs::read(&file_path).unwrap_or_else(|_| Vec::from(PAGE404))
                    }
                };

                let content_length = file.len();

                let mut resp = ResponseHeader::build(status, Some(4))?;
                resp.insert_header(header::SERVER, "Pingpong")?;
                resp.insert_header(header::CONTENT_LENGTH, content_length.to_string())?;
                resp.insert_header(header::CONTENT_TYPE, get_mime_type(&file_path))?;
                session.write_response_header(Box::new(resp), false).await?;

                session.write_response_body(Some(file.into()), true).await?;
                return Ok(true);
            }
        }

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

        if let Some(sni) = &ctx.sni {
            if let Some(s) = &ctx.source {
                let source: &Source = self.routes.get(sni).unwrap().get(s).unwrap();

                if let Some(heads) = &source.headers_response_as_ref() {
                    for head in heads {
                        upstream_response.insert_header(String::from(head.0), head.1)?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn fail_to_proxy(&self, session: &mut Session, _e: &Error, _ctx: &mut Self::CTX) -> u16
    where
        Self::CTX: Send + Sync,
    {
        make_page50x(session, StatusCode::BAD_GATEWAY).await.unwrap();
        502
    }
}

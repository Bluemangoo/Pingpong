use pingora::{Error, HTTPStatus};
use std::collections::HashMap;
use std::path::Path;

use crate::config::{Location, Proxy, Source, StaticServer};
use crate::gateway::GatewayCTX;
use crate::util::path;

pub fn match_route(uri: &str, source: &Source) -> bool {
    let location = source.location_as_ref();
    for loc in location {
        if match loc {
            Location::Start(l) => uri.starts_with(l),
            Location::Equal(l) => uri.eq(l),
            Location::Regex(re) => re.is_match(uri),
        } {
            return true;
        }
    }
    false
}

pub fn find_route_with_start<'a>(
    sni: &'a str,
    uri: &str,
    routes: &'a HashMap<String, HashMap<String, Source>>,
    depth: usize,
    ctx: &mut GatewayCTX,
    starts_from: (&'a String, &'a Source),
) -> pingora::Result<((&'a String, &'a Source), String)> {
    let mut uri = String::from(uri);
    let mut result: Option<pingora::Result<((&'a String, &'a Source), String)>> = None;
    if let Some(rewrites) = &starts_from.1.rewrite_as_ref() {
        for rewrite in rewrites {
            if result.is_some() {
                break;
            }
            if rewrite.regex_as_ref().is_match(&uri) {
                uri = rewrite
                    .regex_as_ref()
                    .replace_all(&uri, rewrite.replace_as_ref())
                    .to_string();

                if rewrite.is_last() {
                    result = Some(find_route(sni, &uri, routes, depth + 1, ctx));
                    break;
                }
            }
        }
    }
    match result {
        None => Ok(((starts_from.0, starts_from.1), String::from(&uri))),
        Some(result) => result,
    }
}

pub fn find_route<'a>(
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
    if let Some(sni_sources) = routes.get(&sni.to_lowercase()) {
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

pub fn check_proxy_status(source: &Proxy) -> bool {
    if let Some(lb) = &source.load_balancer {
        return lb.select(b"", 256).is_some();
    }
    true
}

pub fn check_static_status(source: &StaticServer, path: &str) -> bool {
    let path = path.split('?').collect::<Vec<&str>>()[0];
    let path = if path.ends_with('/') {
        format!("{}index.html", path)
    } else {
        path.to_string()
    };
    Path::new(&path::resolve_uri(&source.root, &path)).exists()
}

pub fn check_status(source: &Source, path: &str) -> bool {
    match source {
        Source::Proxy(proxy) => check_proxy_status(proxy),
        Source::Static(static_server) => check_static_status(static_server, path),
    }
}
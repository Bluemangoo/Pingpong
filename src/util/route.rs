use std::collections::HashMap;
use pingora::{Error, HTTPStatus};
use crate::config::{Location, RewriteFlag, Source};
use crate::gateway::GatewayCTX;

pub fn match_route(uri: &str, source: &Source) -> bool {
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
    if let Some(rewrites) = &starts_from.1.rewrite {
        for rewrite in rewrites {
            if result.is_some() {
                break;
            }
            if rewrite.0.is_match(&uri) {
                uri = rewrite.0.replace_all(&uri, &rewrite.1).to_string();
                if let RewriteFlag::Last = rewrite.2 {
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

pub fn check_status(source: &Source) -> bool {
    if let Some(lb) = &source.load_balancer {
        if lb.select(b"", 256).is_some() {
            return true;
        }
    }
    false
}
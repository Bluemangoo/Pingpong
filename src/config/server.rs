use crate::config::Importable;
use anyhow::anyhow;
use pingora::lb::health_check;
use pingora::prelude::{background_service, LoadBalancer, RoundRobin};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct Source {
    pub ip: String,
    pub host: Option<String>,
    pub port: u16,
    pub ssl: bool,
    pub load_balancer: Option<Arc<LoadBalancer<RoundRobin>>>,
    pub sni: Option<String>,
    pub location: Vec<Location>,
    pub rewrite: Option<Vec<(Regex, String, RewriteFlag)>>,
    pub fallback: Vec<String>,
    pub headers_request: Option<HashMap<String, String>>,
    pub headers_response: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct SourceRaw {
    pub ip: String,
    pub host: Option<String>,
    pub port: u16,
    pub ssl: bool,
    pub sni: Option<String>,
    pub location: Option<Vec<String>>,
    pub rewrite: Option<Vec<String>>,
    pub fallback: Option<Vec<String>>,
    pub headers_request: Option<Importable<HashMap<String, String>>>,
    pub headers_response: Option<Importable<HashMap<String, String>>>,
}

#[derive(Deserialize, Clone)]
pub struct Ssl {
    pub cert: String,
    pub key: String,
}

#[derive(Clone)]
pub struct Server {
    pub source: HashMap<String, Source>,
    pub ssl: Option<Ssl>,
    pub threads: Option<usize>,
    pub check_status: bool,
    pub check_duration: u64,
}

#[derive(Deserialize)]
pub struct ServerRaw {
    pub source: Importable<HashMap<String, Importable<SourceRaw>>>,
    pub ssl: Option<Ssl>,
    pub threads: Option<usize>,
    pub check_status: Option<bool>,
    pub check_duration: Option<u64>,
}

#[derive(Clone)]
pub enum RewriteFlag {
    Last,
    Break,
}

#[derive(Clone)]
pub enum Location {
    Start(String),
    Equal(String),
    Regex(Regex),
}

impl Server {
    pub fn from_raw(raw: ServerRaw, path: &str) -> anyhow::Result<Self> {
        let (source_raw, source_path) = raw.source.import(path)?;
        let mut source = HashMap::new();
        let check_status = raw.check_status.unwrap_or_default();
        let check_duration = raw.check_duration.unwrap_or(1000);
        for i in source_raw {
            let sr = i.1.import(&source_path)?;
            source.insert(
                i.0,
                Source::from_raw(sr.0, &sr.1, check_status, check_duration)?,
            );
        }
        Ok(Self {
            source,
            ssl: raw.ssl,
            threads: raw.threads,
            check_status,
            check_duration,
        })
    }
}

fn into_rewrite(
    parts: (&str, &str),
    path: &str,
    flag: RewriteFlag,
) -> anyhow::Result<(Regex, String, RewriteFlag)> {
    Ok((
        Regex::new(parts.0).map_err(|err| anyhow!("{} {}", path, err.to_string()))?,
        String::from(parts.1),
        flag,
    ))
}

impl Source {
    pub fn from_raw(
        raw: SourceRaw,
        path: &str,
        check_status: bool,
        check_duration: u64,
    ) -> anyhow::Result<Self> {
        let headers_request = match raw.headers_request {
            Some(h) => Some(h.import(path)?.0),
            None => None,
        };
        let headers_response = match raw.headers_response {
            Some(h) => Some(h.import(path)?.0),
            None => None,
        };
        let load_balancer = if check_status {
            let mut upstreams: LoadBalancer<RoundRobin> =
                LoadBalancer::try_from_iter([format!("{}{}", &raw.ip, &raw.port)]).unwrap();
            let hc = health_check::TcpHealthCheck::new();
            upstreams.set_health_check(hc);
            upstreams.health_check_frequency = Some(Duration::from_millis(check_duration));
            Some(background_service("health check", upstreams).task())
        } else {
            None
        };
        let location = match raw.location {
            None => vec![Location::Start(String::from("/"))],
            Some(loc) => {
                let mut result: Vec<Location> = Vec::new();
                for location in loc {
                    result.push(if location.starts_with('/') {
                        Location::Start(location)
                    } else {
                        let parts = location.split(' ').collect::<Vec<&str>>();
                        if parts.len() != 1 {
                            Err(anyhow!("{} Wrong syntax: location = {}", path, location))?;
                        }
                        if parts[0] == "^" {
                            Location::Start(if parts[1].ends_with('/') {
                                String::from(parts[1])
                            } else {
                                format!("{}{}", parts[1], '/')
                            })
                        } else if parts[0] == "=" {
                            Location::Equal(String::from(parts[1]))
                        } else if parts[0] == "~" {
                            Location::Regex(
                                Regex::new(parts[1])
                                    .map_err(|err| anyhow!("{} {}", path, err.to_string()))?,
                            )
                        } else {
                            Err(anyhow!("{} Wrong syntax: location = {}", path, location))?
                        }
                    })
                }
                result
            }
        };
        let rewrite = match raw.rewrite {
            None => None,
            Some(list) => Some({
                let mut vec: Vec<(Regex, String, RewriteFlag)> = Vec::new();
                list.iter().try_for_each(|v| {
                    let parts = v.split(' ').collect::<Vec<&str>>();
                    let res = if parts.len() == 2 {
                        into_rewrite((parts[0], parts[1]), path, RewriteFlag::Last)
                    } else if parts.len() == 3 {
                        if parts[2] == "last" {
                            into_rewrite((parts[0], parts[1]), path, RewriteFlag::Last)
                        } else if parts[2] == "break" {
                            into_rewrite((parts[0], parts[1]), path, RewriteFlag::Break)
                        } else {
                            Err(anyhow!("{} Unknown rewrite flag {}", path, parts[3]))
                        }
                    } else {
                        Err(anyhow!("{} Wrong syntax in rewrite: {}", path, v))
                    };
                    match res {
                        Ok(v) => {
                            vec.push(v);
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                })?;
                vec
            }),
        };
        Ok(Self {
            ip: raw.ip,
            host: raw.host,
            port: raw.port,
            ssl: raw.ssl,
            load_balancer,
            sni: raw.sni,
            location,
            rewrite,
            fallback: raw.fallback.unwrap_or_default(),
            headers_request,
            headers_response,
        })
    }
}

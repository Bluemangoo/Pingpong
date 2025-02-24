use crate::config::{Importable, Location, Rewrite, Source, SourceRaw};
use pingora::lb::health_check;
use pingora::prelude::{background_service, LoadBalancer, RoundRobin};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct Proxy {
    pub ip: String,
    pub host: Option<String>,
    pub port: u16,
    pub ssl: bool,
    pub load_balancer: Option<Arc<LoadBalancer<RoundRobin>>>,
    pub sni: Option<String>,
    pub location: Vec<Location>,
    pub rewrite: Option<Vec<Rewrite>>,
    pub fallback: Vec<String>,
    pub headers_request: Option<HashMap<String, String>>,
    pub headers_response: Option<HashMap<String, String>>,
}

impl Debug for Proxy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proxy")
            .field("ip", &self.ip)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("ssl", &self.ssl)
            .field("load_balancer", &self.load_balancer.is_some())
            .field("sni", &self.sni)
            .field("location", &self.location)
            .field("rewrite", &self.rewrite)
            .field("fallback", &self.fallback)
            .field("headers_request", &self.headers_request)
            .field("headers_response", &self.headers_response)
            .finish()
    }
}

#[derive(Deserialize)]
pub struct ProxyRaw {
    pub source_type: Option<String>,
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

#[derive(Deserialize, Clone, Debug)]
pub struct Ssl {
    pub cert: String,
    pub key: String,
}

#[derive(Clone, Debug)]
pub struct Server {
    pub source: HashMap<String, Source>,
    pub ssl: Option<Ssl>,
    pub threads: Option<usize>,
    pub check_status: bool,
    // pub check_duration: u64,
}

#[derive(Deserialize)]
pub struct ServerRaw {
    pub source: Importable<HashMap<String, Importable<SourceRaw>>>,
    pub ssl: Option<Ssl>,
    pub threads: Option<usize>,
    pub check_status: Option<bool>,
    pub check_duration: Option<u64>,
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
            // check_duration,
        })
    }
}

impl Proxy {
    pub fn from_raw(
        raw: ProxyRaw,
        path: &str,
        check_status: bool,
        check_duration: u64,
    ) -> anyhow::Result<Self> {
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
        let sni = raw.sni.map(|s| s.to_lowercase());
        let location = match raw.location {
            None => vec![Location::Start(String::from("/"))],
            Some(loc) => {
                let mut result: Vec<Location> = Vec::new();
                for location in loc {
                    result.push(Location::new(location, path)?)
                }
                result
            }
        };
        let rewrite = match raw.rewrite {
            None => None,
            Some(list) => Some({
                let mut vec: Vec<Rewrite> = Vec::new();
                list.iter().try_for_each(|v| -> anyhow::Result<()> {
                    vec.push(Rewrite::new(v.clone(), path)?);
                    Ok(())
                })?;
                vec
            }),
        };
        let headers_request = match raw.headers_request {
            Some(h) => Some(h.import(path)?.0),
            None => None,
        };
        let headers_response = match raw.headers_response {
            Some(h) => Some(h.import(path)?.0),
            None => None,
        };
        Ok(Self {
            ip: raw.ip,
            host: raw.host,
            port: raw.port,
            ssl: raw.ssl,
            load_balancer,
            sni,
            location,
            rewrite,
            fallback: raw.fallback.unwrap_or_default(),
            headers_request,
            headers_response,
        })
    }
}

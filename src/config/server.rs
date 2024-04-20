use crate::config::Importable;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Source {
    pub ip: String,
    pub host: Option<String>,
    pub port: u16,
    pub ssl: bool,
    pub sni: Option<String>,
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
}

#[derive(Deserialize)]
pub struct ServerRaw {
    pub source: Importable<HashMap<String, Importable<SourceRaw>>>,
    pub ssl: Option<Ssl>,
    pub threads: Option<usize>,
}

impl Server {
    pub fn from_raw(raw: ServerRaw, path: &str) -> anyhow::Result<Self> {
        let (source_raw, source_path) = raw.source.import(path)?;
        let mut source = HashMap::new();
        for i in source_raw {
            let sr = i.1.import(&source_path)?;
            source.insert(i.0, Source::from_raw(sr.0, &sr.1)?);
        }
        Ok(Self {
            source,
            ssl: raw.ssl,
            threads: raw.threads,
        })
    }
}

impl Source {
    pub fn from_raw(raw: SourceRaw, path: &str) -> anyhow::Result<Self> {
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
            sni: raw.sni,
            headers_request,
            headers_response,
        })
    }
}

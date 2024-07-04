use crate::config::{Location, Proxy, ProxyRaw, Rewrite, StaticServer, StaticServerRaw};
use serde::de::{Error, IntoDeserializer};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use toml::Value;

pub enum SourceRaw {
    Proxy(ProxyRaw),
    Static(StaticServerRaw),
}

#[derive(Deserialize)]
struct SourceMatch {
    pub source_type: Option<String>,
}

fn merge_err(e: Option<toml::de::Error>, e2: toml::de::Error) -> toml::de::Error {
    match e {
        None => e2,
        Some(e) => toml::de::Error::custom(format!("{}\n{}", e.message(), e2.message())),
    }
}

impl<'de> Deserialize<'de> for SourceRaw {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let mat = SourceMatch::deserialize(value.clone().into_deserializer())
            .unwrap()
            .source_type
            .map(|v| v.to_lowercase());
        let v = ProxyRaw::deserialize(value.clone().into_deserializer());
        let mut err: Option<toml::de::Error> = None;
        match v {
            Ok(i) => {
                match &i.source_type {
                    None => {
                        return Ok(SourceRaw::Proxy(i));
                    }
                    Some(s) => {
                        if s.to_lowercase() == "proxy" {
                            return Ok(SourceRaw::Proxy(i));
                        }
                    }
                };
            }
            Err(e) => {
                match &mat {
                    None => err = Some(merge_err(err, e)),
                    Some(v) => {
                        if v == "proxy" {
                            err = Some(merge_err(err, e))
                        }
                    }
                };
            }
        };
        let v = StaticServerRaw::deserialize(value.clone().into_deserializer());
        match v {
            Ok(i) => {
                match &i.source_type {
                    None => {
                        return Ok(SourceRaw::Static(i));
                    }
                    Some(s) => {
                        if s.to_lowercase() == "static" {
                            return Ok(SourceRaw::Static(i));
                        }
                    }
                };
            }
            Err(e) => {
                match &mat {
                    None => err = Some(merge_err(err, e)),
                    Some(v) => {
                        if v == "static" {
                            err = Some(merge_err(err, e))
                        }
                    }
                };
            }
        };
        return match err {
            None => Err(Error::custom("unknown source type")),
            Some(err) => Err(Error::custom(err.message())),
        };
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum Source {
    Proxy(Proxy),
    Static(StaticServer),
}

#[allow(dead_code)]
impl Source {
    pub fn from_raw(
        raw: SourceRaw,
        path: &str,
        check_status: bool,
        check_duration: u64,
    ) -> Result<Self, anyhow::Error> {
        match raw {
            SourceRaw::Proxy(i) => Ok(Source::Proxy(Proxy::from_raw(
                i,
                path,
                check_status,
                check_duration,
            )?)),
            SourceRaw::Static(i) => Ok(Source::Static(StaticServer::from_raw(i, path)?)),
        }
    }

    pub fn sni_as_ref(&self) -> &Option<String> {
        match self {
            Source::Proxy(p) => &p.sni,
            Source::Static(s) => &s.sni,
        }
    }

    pub fn location_as_ref(&self) -> &Vec<Location> {
        match self {
            Source::Proxy(p) => &p.location,
            Source::Static(s) => &s.location,
        }
    }

    pub fn rewrite_as_ref(&self) -> &Option<Vec<Rewrite>> {
        match self {
            Source::Proxy(p) => &p.rewrite,
            Source::Static(s) => &s.rewrite,
        }
    }

    pub fn fallback_as_ref(&self) -> &Vec<String> {
        match self {
            Source::Proxy(p) => &p.fallback,
            Source::Static(s) => &s.fallback,
        }
    }

    pub fn headers_request_as_ref(&self) -> &Option<HashMap<String, String>> {
        match self {
            Source::Proxy(p) => &p.headers_request,
            Source::Static(s) => &s.headers_request,
        }
    }

    pub fn headers_response_as_ref(&self) -> &Option<HashMap<String, String>> {
        match self {
            Source::Proxy(p) => &p.headers_response,
            Source::Static(s) => &s.headers_response,
        }
    }

    pub fn is_proxy(&self) -> bool {
        match self {
            Source::Proxy(_) => true,
            Source::Static(_) => false,
        }
    }

    pub fn is_static(&self) -> bool {
        match self {
            Source::Proxy(_) => false,
            Source::Static(_) => true,
        }
    }
}

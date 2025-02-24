use crate::config::{Importable, Location, Rewrite};
use crate::util::path;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct StaticServer {
    pub root: String,
    pub sni: Option<String>,
    pub location: Vec<Location>,
    pub rewrite: Option<Vec<Rewrite>>,
    pub fallback: Vec<String>,
    pub headers_request: Option<HashMap<String, String>>,
    pub headers_response: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
pub struct StaticServerRaw {
    pub source_type: Option<String>,
    pub root: String,
    pub sni: Option<String>,
    pub location: Option<Vec<String>>,
    pub rewrite: Option<Vec<String>>,
    pub fallback: Option<Vec<String>>,
    pub headers_request: Option<Importable<HashMap<String, String>>>,
    pub headers_response: Option<Importable<HashMap<String, String>>>,
}

impl StaticServer {
    pub fn from_raw(raw: StaticServerRaw, path: &str) -> Result<Self, anyhow::Error> {
        let root = path::resolve(
            path,
            &if raw.root.ends_with('/') {
                raw.root
            } else {
                format!("{}/", raw.root)
            },
        );
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
            root,
            sni,
            location,
            rewrite,
            fallback: raw.fallback.unwrap_or_default(),
            headers_request,
            headers_response,
        })
    }
}

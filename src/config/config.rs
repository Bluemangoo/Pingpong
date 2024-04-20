use crate::config::{Importable, Server, ServerRaw};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct Log {
    pub access: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct Config {
    pub server: HashMap<String, Server>,
    pub log: Log,
}

#[derive(Deserialize)]
pub struct ConfigRaw {
    pub server: Importable<HashMap<String, Importable<ServerRaw>>>,
    pub log: Log,
}

impl Config {
    pub fn from_raw(raw: ConfigRaw, path: &str) -> anyhow::Result<Self> {
        let (server_raw, server_path) = raw.server.import(path)?;
        let mut server = HashMap::new();
        for i in server_raw {
            let (sr, sr_path) = i.1.import(&server_path)?;
            server.insert(i.0, Server::from_raw(sr, &sr_path)?);
        }
        Ok(Self {
            server,
            log: raw.log,
        })
    }
}

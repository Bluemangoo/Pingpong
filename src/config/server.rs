use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Source {
    pub ip: String,
    pub host: Option<String>,
    pub port: u16,
    pub ssl: bool,
    pub sni: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Ssl {
    pub cert: String,
    pub key: String,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub source: HashMap<String, Source>,
    pub ssl: Option<Ssl>,
    pub threads: Option<usize>
}
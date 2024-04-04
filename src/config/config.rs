use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Proxy {
    pub file: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Log {
    pub access: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub proxy: Proxy,
    pub log: Log,
}

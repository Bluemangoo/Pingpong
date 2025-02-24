use crate::config::{Importable, Server, ServerRaw};
use pingora::server::configuration::ServerConf;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug)]
pub struct Config {
    pub server: HashMap<String, Server>,
    pub log: Option<String>,
    pub version: Option<usize>,
    pub pid_file: Option<String>,
    pub upgrade_sock: Option<String>,
    pub user: Option<String>,
    pub group: Option<String>,
    pub threads: Option<usize>,
    pub work_stealing: Option<bool>,
    pub ca_file: Option<String>,
    pub grace_period_seconds: Option<u64>,
    pub graceful_shutdown_timeout_seconds: Option<u64>,
    pub client_bind_to_ipv4: Option<Vec<String>>,
    pub client_bind_to_ipv6: Option<Vec<String>>,
    pub upstream_connect_offload_threadpools: Option<usize>,
    pub upstream_connect_offload_thread_per_pool: Option<usize>,
}

#[derive(Deserialize)]
pub struct ConfigRaw {
    pub server: Importable<HashMap<String, Importable<ServerRaw>>>,
    pub log: Option<String>,
    pub version: Option<usize>,
    pub pid_file: Option<String>,
    pub upgrade_sock: Option<String>,
    pub user: Option<String>,
    pub group: Option<String>,
    pub threads: Option<usize>,
    pub work_stealing: Option<bool>,
    pub ca_file: Option<String>,
    pub grace_period_seconds: Option<u64>,
    pub graceful_shutdown_timeout_seconds: Option<u64>,
    pub client_bind_to_ipv4: Option<Vec<String>>,
    pub client_bind_to_ipv6: Option<Vec<String>>,
    pub upstream_connect_offload_threadpools: Option<usize>,
    pub upstream_connect_offload_thread_per_pool: Option<usize>,
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
            pid_file: raw.pid_file,
            log: raw.log,
            version: raw.version,
            upgrade_sock: raw.upgrade_sock,
            user: raw.user,
            group: raw.group,
            threads: raw.threads,
            work_stealing: raw.work_stealing,
            ca_file: raw.ca_file,
            grace_period_seconds: raw.grace_period_seconds,
            graceful_shutdown_timeout_seconds: raw.graceful_shutdown_timeout_seconds,
            client_bind_to_ipv4: raw.client_bind_to_ipv4,
            client_bind_to_ipv6: raw.client_bind_to_ipv6,
            upstream_connect_offload_threadpools: raw.upstream_connect_offload_threadpools,
            upstream_connect_offload_thread_per_pool: raw.upstream_connect_offload_thread_per_pool,
        })
    }

    pub fn merge_into_pingora_config(&self, base_config: &mut Arc<ServerConf>) {
        let base_config = Arc::get_mut(base_config).unwrap();
        if let Some(version) = self.version {
            base_config.version = version.clone();
        };
        if let Some(pid_file) = &self.pid_file {
            base_config.pid_file = pid_file.clone();
        };
        if let Some(upgrade_sock) = &self.upgrade_sock {
            base_config.upgrade_sock = upgrade_sock.clone();
        };
        if self.user.is_some() {
            base_config.user = self.user.clone();
        };
        if self.group.is_some() {
            base_config.group = self.group.clone();
        };
        if let Some(threads) = &self.threads {
            base_config.threads = threads.clone();
        };
        if let Some(work_stealing) = &self.work_stealing {
            base_config.work_stealing = work_stealing.clone();
        };
        if self.ca_file.is_some() {
            base_config.ca_file = self.ca_file.clone();
        };
        if self.grace_period_seconds.is_some() {
            base_config.grace_period_seconds = self.grace_period_seconds.clone();
        };
        if self.graceful_shutdown_timeout_seconds.is_some() {
            base_config.graceful_shutdown_timeout_seconds =
                self.graceful_shutdown_timeout_seconds.clone();
        };
        if let Some(client_bind_to_ipv4) = &self.client_bind_to_ipv4 {
            base_config.client_bind_to_ipv4 = client_bind_to_ipv4.clone();
        };
        if let Some(client_bind_to_ipv6) = &self.client_bind_to_ipv6 {
            base_config.client_bind_to_ipv6 = client_bind_to_ipv6.clone();
        };
        if self.upstream_connect_offload_threadpools.is_some() {
            base_config.upstream_connect_offload_threadpools =
                self.upstream_connect_offload_threadpools.clone();
        };
        if self.upstream_connect_offload_thread_per_pool.is_some() {
            base_config.upstream_connect_offload_thread_per_pool =
                self.upstream_connect_offload_thread_per_pool.clone();
        };
    }
}

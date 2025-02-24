mod config;
mod gateway;
mod util;

use crate::config::Importable;
use crate::gateway::Gateway;
use crate::util::path;
use anyhow::anyhow;
use log::debug;
use pingora::prelude::*;
use simplelog::*;
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::path::Path;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(short, long)]
    pub upgrade: bool,
    #[structopt(short, long)]
    pub daemon: bool,
    #[structopt(long)]
    pub nocapture: bool,
    #[structopt(short, long)]
    pub test: bool,
}

impl From<Opt> for Option<pingora::prelude::Opt> {
    fn from(opt: Opt) -> Self {
        Some(pingora::prelude::Opt {
            upgrade: opt.upgrade,
            daemon: opt.daemon,
            nocapture: opt.nocapture,
            test: opt.test,
            conf: None,
        })
    }
}

#[derive(StructOpt)]
struct CommandOpt {
    #[structopt(short = "c")]
    config: Option<String>,

    #[structopt(long)]
    pub debug: bool,
    #[structopt(short, long)]
    pub version: bool,

    #[structopt(flatten)]
    base_opts: Opt,
}

fn main() -> anyhow::Result<()> {
    let command_opts = CommandOpt::from_args();

    if command_opts.version {
        println!("Pingpong {}+{}", env!("CARGO_PKG_VERSION"), env!("GIT_REF"));
        return Ok(());
    }

    let base = env::current_exe()?;
    let base = base.to_str().unwrap();

    let config_base: Importable<config::ConfigRaw> =
        Importable::Import(match command_opts.config {
            None => {
                if Path::new(&path::resolve(base, "config/pingpong.toml")).exists() {
                    String::from("config/pingpong.toml")
                } else {
                    String::from("/etc/pingpong/pingpong.toml")
                }
            }
            Some(c) => c,
        });
    let config: config::Config = {
        let c = config_base.import(base)?;
        config::Config::from_raw(c.0, &c.1)?
    };

    let log_level = if command_opts.debug {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    match match &config.log {
        None => None,
        Some(path) => {
            let path = path::resolve(env::current_exe()?.to_str().unwrap(), path);
            match path::create(&path) {
                Ok(_) => Some(path),
                Err(error) => {
                    println!("{}", error);
                    println!("Failed to init logger, fallback to terminal.");
                    None
                }
            }
        }
    } {
        Some(path) => WriteLogger::init(
            log_level,
            Config::default(),
            OpenOptions::new().append(true).open(path)?,
        )?,
        None => TermLogger::init(
            log_level,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )?,
    };

    let mut server = Server::new(command_opts.base_opts)?;

    config.merge_into_pingora_config(&mut server.configuration);

    server.bootstrap();

    debug!("Pingpong bootstrapping");
    for i in config.server {
        let port = u16::from_str(&i.0)?;
        debug!("Loading server on port {}", port);
        let mut service_config: HashMap<String, HashMap<String, config::Source>> = HashMap::new();

        for source in i.1.source {
            debug!("Loading source {}", source.0);
            debug!("Source {}: {:?}", source.0, source.1);
            let sni = match &source.1.sni_as_ref() {
                None => "",
                Some(sni) => sni,
            };
            if !service_config.contains_key(sni) {
                service_config.insert(String::from(sni), HashMap::new());
            }
            service_config
                .get_mut(sni)
                .unwrap()
                .insert(source.0.clone(), source.1);
            debug!("Source {} loaded", source.0);
        }
        let mut service = http_proxy_service(
            &server.configuration,
            Gateway::new(port, service_config, i.1.check_status),
        );

        match i.1.threads {
            None => {}
            Some(threads) => service.threads = Some(threads),
        };

        match i.1.ssl {
            None => {
                debug!("ssl disabled");
                service.add_tcp(&format!("0.0.0.0:{}", port));
            }
            Some(ssl) => {
                debug!("ssl enabled");
                service
                    .add_tls(&format!("0.0.0.0:{}", port), &ssl.cert, &ssl.key)
                    .or(Err(anyhow!(
                        "Failed to read cert:\n{}\n{}",
                        &ssl.cert,
                        &ssl.key
                    )))?;
            }
        }

        server.add_service(service);
        debug!("Server on port {} loaded", port);
    }
    debug!("Pingpong bootstrapped");

    server.run_forever()
}

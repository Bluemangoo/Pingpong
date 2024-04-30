mod config;
mod gateway;
mod util;

use crate::config::Importable;
use crate::gateway::Gateway;
use crate::util::path;
use anyhow::anyhow;
use log::error;
use pingora::prelude::*;
use simplelog::*;
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
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
    #[structopt(short = "c", default_value = "config/pingpong.toml")]
    config: String,

    #[structopt(flatten)]
    base_opts: Opt,
}

fn main() -> anyhow::Result<()> {
    let command_opts = CommandOpt::from_args();

    let config_base: Importable<config::ConfigRaw> = Importable::Import(command_opts.config);
    let config: config::Config = {
        let c = config_base.import(env::current_exe()?.to_str().unwrap())?;
        config::Config::from_raw(c.0, &c.1)?
    };

    #[inline(always)]
    fn create_log(
        level: LevelFilter,
        path: &Option<String>,
    ) -> anyhow::Result<Box<dyn SharedLogger>> {
        let path = match path {
            None => None,
            Some(path) => {
                let path = path::resolve(env::current_exe()?.to_str().unwrap(), path);
                match path::create(&path) {
                    Ok(_) => Some(path),
                    Err(error) => {
                        println!("{}", error);
                        println!(
                            "Failed to init {} logger, fallback to terminal.",
                            level.as_str()
                        );
                        None
                    }
                }
            }
        };
        match path {
            Some(path) => Ok(WriteLogger::new(
                level,
                Config::default(),
                OpenOptions::new().append(true).open(path)?,
            )),
            None => Ok(TermLogger::new(
                level,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            )),
        }
    }

    CombinedLogger::init(vec![
        create_log(LevelFilter::Info, &config.log.access)?,
        create_log(LevelFilter::Error, &config.log.error)?,
    ])?;

    let mut server = Server::new(command_opts.base_opts)?;

    config.merge_into_pingora_config(&mut server.configuration);

    server.bootstrap();

    for i in config.server {
        let port = u16::from_str(&i.0)?;
        let mut service_config: HashMap<String, (String, config::Source)> = HashMap::new();

        for source in i.1.source {
            match &source.1.sni {
                None => {
                    if service_config.contains_key("") {
                        error!(
                            "[{}] Default server was used by {}, ignored.",
                            &i.0,
                            service_config.get("").unwrap().0
                        );
                        continue;
                    }
                    service_config.insert(String::from(""), source);
                }
                Some(sni) => {
                    if service_config.contains_key(sni) {
                        error!(
                            "[{}] Sni conflict! {} was used by {}, ignored.",
                            &i.0,
                            sni,
                            service_config.get(sni).unwrap().0
                        );
                        continue;
                    }
                    service_config.insert(sni.clone(), source);
                }
            }
        }
        let mut service =
            http_proxy_service(&server.configuration, Gateway::new(port, service_config));

        match i.1.threads {
            None => {}
            Some(threads) => service.threads = Some(threads),
        };

        match i.1.ssl {
            None => {
                service.add_tcp(&format!("0.0.0.0:{}", port));
            }
            Some(ssl) => {
                service
                    .add_tls(&format!("0.0.0.0:{}", port), &ssl.cert, &ssl.key)
                    .or(Err(anyhow!(
                        "Failed to read cert:\n{}\n{}",
                        &ssl.cert,
                        &ssl.key
                    )))?;
            }
        }

        server.add_service(service)
    }

    server.run_forever();
    Ok(())
}

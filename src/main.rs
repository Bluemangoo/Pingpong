mod config;
mod gateway;
mod util;

use crate::gateway::Gateway;
use crate::util::path;
use config::*;
use log::{error, info};
use pingora::prelude::*;
use simplelog::*;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt)]
struct CommandOpt {
    #[structopt(short = "i", default_value = "config/pingpong.toml")]
    config: String,

    #[structopt(flatten)]
    base_opts: Opt,
}

fn main() {
    let command_opts = CommandOpt::from_args();

    let config: config::config::Config = {
        let mut file = File::open(&command_opts.config).expect("Failed to open file");
        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)
            .expect("Failed to read file");
        toml::from_str(toml_str.as_str()).unwrap()
    };

    let servers: HashMap<String, server::Server> = {
        let default_path = "proxy.toml".to_string();
        let rel_file_name = match &config.proxy.file {
            None => &default_path,
            Some(n) => n,
        };
        let path = &path::resolve(&command_opts.config, &rel_file_name);
        let mut file = File::open(&path).expect("Failed to open file");
        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)
            .expect("Failed to read file");
        toml::from_str(toml_str.as_str()).unwrap()
    };

    #[inline(always)]
    fn create_log(level: LevelFilter, path: &Option<String>) -> Box<dyn SharedLogger> {
        let path = match path {
            None => None,
            Some(path) => {
                let path = path::resolve(env::current_exe().unwrap().to_str().unwrap(), path);
                match path::create(&path) {
                    Ok(_) => Some(path),
                    Err(error) => {
                        println!("{}", error.to_string());
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
            Some(path) => WriteLogger::new(level, Config::default(), File::create(path).unwrap()),
            None => TermLogger::new(
                level,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
        }
    }

    CombinedLogger::init(vec![
        create_log(LevelFilter::Info, &config.log.access),
        create_log(LevelFilter::Error, &config.log.error),
    ])
    .unwrap();

    let mut server = Server::new(Some(command_opts.base_opts)).unwrap();

    info!("Configuration: {:?}", server.configuration);

    server.bootstrap();

    for i in servers {
        let port = u16::from_str(&i.0).unwrap();
        let mut service_config: HashMap<String, (String, server::Source)> = HashMap::new();

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
                    .expect(format!("Failed to read cert:\n{}\n{}", &ssl.cert, &ssl.key).as_str());
            }
        }

        server.add_service(service)
    }

    server.run_forever();
}

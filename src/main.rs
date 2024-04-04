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
use std::fs::File;
use std::io::Read;
use std::path::Path;
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
        println!("{}", path);
        let mut file = File::open(Path::new(&path)).expect("Failed to open file");
        let mut toml_str = String::new();
        file.read_to_string(&mut toml_str)
            .expect("Failed to read file");
        toml::from_str(toml_str.as_str()).unwrap()
    };

    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(match &config.log.access {
                None => "/var/log/pingpong/access.log",
                Some(path) => path,
            })
            .unwrap(),
        ),
        WriteLogger::new(
            LevelFilter::Error,
            Config::default(),
            File::create(match &config.log.error {
                None => "/var/log/pingpong/error.log",
                Some(path) => path,
            })
            .unwrap(),
        ),
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

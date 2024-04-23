# Pingpong

Reverse proxy powered by [Pingora](https://github.com/cloudflare/pingora).

> Pingpong. Doesn't the little bouncing-around ball resembles your data packets tossed around among NATs and ISPs?

## Installation and Usage

Pingora only support Linux and Mac, and Windows won't be supported.

- Download archive file from [release](https://github.com/cloudflare/pingora), and unpack it somewhere.
- Modify the config file.
- Run with `pingpong -c /path/to/pingpong.toml`. If config location isn't specified, Pingpong will use `./config/pingpong.toml` based on executable's path.

## Commandline Arguments

- `-i`: The path to the configuration file (of Pingpong).

Followings are for Pingora:

- `-u` or `--upgrade`: Whether this server should try to upgrade from a running old server.
- `-d` or `--daemon`: Whether to run this server in the background.
- `-t` or `--test`: Test the configuration (of Pingora) and exit.

## Config

Here is configuration for Pingpong. See Pingora's [here](https://github.com/cloudflare/pingora/blob/main/docs/user_guide/conf.md).

Pingpong use toml for its config. Import is pioneeringly allowed. For importable item, you can use `import = path/to/another/file.toml`.

See examples [here](https://github.com/Bluemangoo/Pingpong/tree/master/config).

- `version`: Optional, the version of the config, currently it is a constant 1;
- `pid_file`: Optional, the path to the pid file;
- `upgrade_sock`: Optional, the path to the upgrade socket;
- `threads`: Optional, number of threads per service;
- `user`: Optional, the user the pingora server should be run under after daemonization;
- `group`: Optional, the group the pingora server should be run under after daemonization;
- `client_bind_to_ipv4`: Optional, source IPv4 addresses to bind to when connecting to server;
- `client_bind_to_ipv6`: Optional, source IPv6 addresses to bind to when connecting to server;
- `ca_file`: Optional, The path to the root CA file;
- `work_stealing`: Optional, Enable work stealing runtime (default true). See Pingora runtime (WIP) section for more info;
- `upstream_keepalive_pool_size`: Optional, The number of total connections to keep in the connection pool.
- `log.access`: Optional, The path to the access log file, default to terminal;
- `log.error`: Optional, The path to the error log file, default to terminal;
- `server`: `Map<Port, Server>`, port is filled as a string but will be converted to `u16`. Importable, and each server is also importable.

Properties of Server

- `thread`: Thread for this server.
- `source`: `Map<String, Source>`. Importable, and each source is also importable.

Properties of Source:

- `ip`: Ip of upstream service, instead of domain.
- `port`: Port of upstream service.
- `ssl`: Whether upstream service is on ssl.
- `sni`: Sni for this service. Only request with corresponding sni will be route to this service. It's optional, as the unset one in one server will be the default.
- `host`: Optional, rewrite `Host` in request headers. Fill if upstream service also use sni to recognize route.
- `headers_request`: `Map<String, String>`. Optional and importable, add or replace the header in request.
- `headers_response`: `Map<String, String>`. Optional and importable, add or replace the header in response.

## Build

**You can find the latest build in [Actions](https://github.com/Bluemangoo/Pingpong/actions/workflows/build.yml).**

Makesure you have cargo and rustc installed.

### Build from scratch

```bash
cargo build
```

If successful, you can find the excutable binary here: `target/debug/pingpong`

### Build optimised one

```bash
cargo build --release
```

If successful, you can find the excutable binary here: `target/release/pingpong`

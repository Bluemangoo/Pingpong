# Pingpong

Reverse proxy powered by [Pingora](https://github.com/cloudflare/pingora)

## Installation and Usage

Pingora only support Linux and Mac, and Windows won't be supported.

- Download archive file from [release](https://github.com/cloudflare/pingora), and unpack it somewhere.
- Modify the config file.
- Run with `pingpong -i /path/to/pingpong.toml`. If config location isn't specified, Pingpong will use `./config/pingpong.toml` based on executable's path.

## Commandline Arguments

- `-i`: The path to the configuration file (of Pingpong).

Followings are for Pingora:

- `-u` or `--upgrade`: Whether this server should try to upgrade from a running old server.
- `-d` or `--daemon`: Whether to run this server in the background.
- `-t` or `--test`: Test the configuration (of Pingora) and exit.
- `-c` or `--conf`: The path to the configuration file (of Pingora).

## Config

Here is configuration for Pingpong. See Pingora's [here](https://github.com/cloudflare/pingora/blob/main/docs/user_guide/conf.md).

Pingpong use toml for its config. Import is pioneeringly allowed. For importable item, you can use `import = path/to/another/file.toml`.

See examples [here](https://github.com/Bluemangoo/Pingpong/tree/master/config).

- `log.access`: The path to the access log file;
- `log.error`: The path to the error log file;
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

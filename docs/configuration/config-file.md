---
prev: false
next: false
---

# Config File

If config location isn't specified, Pingpong will use `./config/pingpong.toml` based on executable's path, or `/etc/pingpong/pingpong.toml`.

Here is configuration for Pingpong. See Pingora's [here](https://github.com/cloudflare/pingora/blob/main/docs/user_guide/conf.md).

::: tip
Pingpong use toml for its config. Import is pioneeringly allowed. For importable item, you can use `import = path/to/another/file.toml`.
:::

See examples [here](https://github.com/Bluemangoo/Pingpong/tree/master/config).

## Config Items

- `version`: **Optional**, the version of the config, currently it is a constant 1;
- `pid_file`: **Optional**, the path to the pid file;
- `upgrade_sock`: **Optional**, the path to the upgrade socket;
- `threads`: **Optional**, number of threads per service;
- `user`: **Optional**, the user the pingora server should be run under after daemonization;
- `group`: **Optional**, the group the pingora server should be run under after daemonization;
- `client_bind_to_ipv4`: **Optional**, source IPv4 addresses to bind to when connecting to server;
- `client_bind_to_ipv6`: **Optional**, source IPv6 addresses to bind to when connecting to server;
- `ca_file`: **Optional**, The path to the root CA file;
- `work_stealing`: **Optional**, Enable work stealing runtime (default true). See Pingora runtime (WIP) section for more info;
- `upstream_keepalive_pool_size`: **Optional**, The number of total connections to keep in the connection pool.
- `log`: **Optional**, The path to the log file, default to terminal;
- `server`: `Map<Port, Server>`, **Importable**, port is filled as a string but will be converted to `u16`. See `Server`'s definition [here](../server).

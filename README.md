# Pingpong

Reverse proxy powered by [Pingora](https://github.com/cloudflare/pingora).

> Pingpong. Doesn't the little bouncing-around ball resembles your data packets tossed around among NATs and ISPs?

Read the document [here](https://pingpong.bluemangoo.net)

## Installation and Usage

Pingora only support Linux and Mac, and Windows won't be supported.

- Download archive file from [release](https://github.com/Bluemangoo/Pingpong/releases), and unpack it somewhere.
- Modify the config file.
- Run with `pingpong -c /path/to/pingpong.toml`. If config location isn't specified, Pingpong will use `./config/pingpong.toml` based on executable's path, or `/etc/pingpong/pingpong.toml`.

## Build

**You can find the latest x86_64 build in [Actions](https://github.com/Bluemangoo/Pingpong/actions/workflows/build.yml).**

Make sure you have cargo and rustc installed.

### Build from scratch

```bash
cargo build
```

If successful, you can find the executable binary here: `target/debug/pingpong`

### Build optimised one

```bash
cargo build --release
```

If successful, you can find the executable binary here: `target/release/pingpong`

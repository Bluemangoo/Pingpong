---
prev: 
    text: Introduction
    link: /getting-started/introduction
next: false
---

# Getting Started

## Installing

### Installing from GitHub

Download archive file from [release](https://github.com/Bluemangoo/Pingpong/releases).

Unpack it somewhere. We recommend to keep config file in the same directory, or put it under `/etc/pingpong/`.

### Installing from Aur (Arch Linux)

Pingpong Aur package is maintained by [Serverbread](https://github.com/serverbread)

```bash
paru -S pingpong
```

## Running

The default config file path is `./config/pingpong.toml` based on executable's path, or `/etc/pingpong/pingpong.toml`. If your config file is not here, please specify it with `-c` flag.

```bash
pingpong -c /path/to/pingpong.toml
```

You can register it as a service with systemd.

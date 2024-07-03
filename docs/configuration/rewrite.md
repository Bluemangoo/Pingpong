---
pref: false
next: false
---
# Rewrite

Rewrite syntax is similar to [Nginx](https://nginx.org/r/rewrite), but here you provide a list of rewrite pattern.

Matching is after decoding the text encoded in the “%XX” form. Rewrite up to 10 times.

Syntax: `rewrite-regex URI [flag]`.

An optional `flag` parameter can be one of:
- `last`: The default one, stops processing the current set of rewrite and starts a search for a new location matching the changed URI;
- `break`: stops processing the current rewrite rule and start to search next.

For example:

```toml
rewrite = ["^/(.*) /service2/$1 last"]
```

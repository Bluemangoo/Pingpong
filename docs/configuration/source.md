---
prev: false
next: false
---

# Source

Source is a config class, which configures the source of the server.

It is **importable**.

## Config Items

- `ip`: Ip of upstream service, instead of domain.
- `port`: Port of upstream service.
- `ssl`: Whether upstream service is on ssl.
- `sni`: Sni for this service. Only request with corresponding sni will be route to this service. It's optional, as the unset one in one server will be the default.
- `host`: **Optional**, rewrite `Host` in request headers. Fill if upstream service also use sni to recognize route.
- `headers_request`: `Map<String, String>`. **Optional** and **importable**, add or replace the header in request.
- `headers_response`: `Map<String, String>`. **Optional** and **importable**, add or replace the header in response.
- `location`: **Optional**, default to match all the requests, see [Location](../location).
- `rewrite`: **Optional**, see [Rewrite](../rewrite).
- `fallback`: **Optional**, fallback to other sources when available, only works when `check_status` is enabled. Fallback up to 10 times.
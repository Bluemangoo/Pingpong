---
pref: false
next: false
---

# Server

Server is a config class, which configures the service on **one** port.

It is **importable**.

## Config Items

- `thread`: Thread for this server.
- `source`: `Map<String, Source>`. **Importable**. See `Source`'s definition [here](../source).
- `check_status`: **Optional**, default false, check if source is available, and speedup when unavailable.
- `check_duration`: **Optional**, default 1000, duration of per status check (ms).
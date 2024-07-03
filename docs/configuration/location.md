---
prev: false
next: false
---

# Location

Location syntax is similar to [Nginx](https://nginx.org/r/location), but here you provide a list of location pattern.

Matching is after decoding the text encoded in the “%XX” form.

Syntax: `[ = | ^ | ~ ] URI`.

There are three type: `=`(equal), `^`(startsWith) and `~`(regex).

**There is a space between type and URI.**

When no type is provided and uri is starts with `/`, type will be `^`(startsWith).

*Why the hell a URI will start without `/`??*

For example:

```toml
location = ["/public", "~ /static/*.(gif|jpg|jpeg)"]
```

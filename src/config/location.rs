use anyhow::anyhow;
use regex::Regex;

#[derive(Clone)]
#[derive(Debug)]
pub enum Location {
    Start(String),
    Equal(String),
    Regex(Regex),
}

impl Location {
    pub fn new(location: String, path: &str) -> anyhow::Result<Self> {
        if location.starts_with('/') {
            Ok(Location::Start(location))
        } else {
            let parts = location.split(' ').collect::<Vec<&str>>();
            if parts.len() != 1 {
                Err(anyhow!("{} Wrong syntax: location = {}", path, location))?;
            }
            if parts[0] == "^" {
                Ok(Location::Start(if parts[1].ends_with('/') {
                    String::from(parts[1])
                } else {
                    format!("{}{}", parts[1], '/')
                }))
            } else if parts[0] == "=" {
                Ok(Location::Equal(String::from(parts[1])))
            } else if parts[0] == "~" {
                Ok(Location::Regex(
                    Regex::new(parts[1]).map_err(|err| anyhow!("{} {}", path, err.to_string()))?,
                ))
            } else {
                Err(anyhow!("{} Wrong syntax: location = {}", path, location))?
            }
        }
    }
}

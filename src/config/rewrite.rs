use anyhow::anyhow;
use regex::Regex;

#[derive(Clone)]
pub enum Rewrite {
    Last(Regex, String),
    Break(Regex, String),
}

#[allow(dead_code)]
impl Rewrite {
    pub fn new(rewrite: String, path: &str) -> anyhow::Result<Rewrite> {
        let parts = rewrite.split(' ').collect::<Vec<&str>>();
        if parts.len() != 3 {
            Err(anyhow!("{} Wrong syntax: rewrite = {}", path, rewrite))?;
        }
        let regex = Regex::new(parts[0]).map_err(|err| anyhow!("{} {}", path, err.to_string()))?;
        match parts[2] {
            "last" => Ok(Rewrite::Last(regex, String::from(parts[1]))),
            "break" => Ok(Rewrite::Break(regex, String::from(parts[1]))),
            _ => Err(anyhow!("{} Wrong syntax: rewrite = {}", path, rewrite))?,
        }
    }

    pub fn regex_as_ref(&self) -> &Regex {
        match self {
            Rewrite::Last(re, _) => re,
            Rewrite::Break(re, _) => re,
        }
    }

    pub fn replace_as_ref(&self) -> &str {
        match self {
            Rewrite::Last(_, replace) => replace,
            Rewrite::Break(_, replace) => replace,
        }
    }

    pub fn is_last(&self) -> bool {
        match self {
            Rewrite::Last(_, _) => true,
            Rewrite::Break(_, _) => false,
        }
    }

    pub fn is_break(&self) -> bool {
        match self {
            Rewrite::Last(_, _) => false,
            Rewrite::Break(_, _) => true,
        }
    }
}

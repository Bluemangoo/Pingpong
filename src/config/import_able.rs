use crate::util::path;
use anyhow::anyhow;
use serde::de::{Error, IntoDeserializer};
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::Read;
use toml::Value;

#[derive(Clone)]
pub enum Importable<T> {
    Some(T),
    Import(String),
}

impl<'de, T> Deserialize<'de> for Importable<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Ok(i) = Import::deserialize(value.clone().into_deserializer()) {
            Ok(Importable::Import(i.import))
        } else {
            match T::deserialize(value.into_deserializer()) {
                Ok(v) => Ok(Importable::Some(v)),
                Err(e) => Err(Error::custom(e.message())),
            }
        }
    }
}

impl<T> Importable<T>
where
    T: serde::de::DeserializeOwned,
{
    pub fn import(self, base: &str) -> anyhow::Result<(T, String)> {
        match self {
            Importable::Some(v) => Ok((v, String::from(base))),
            Importable::Import(path) => {
                let path = path::resolve(base, &path);
                let mut file = File::open(&path).or(Err(anyhow!("Cannot read file {}", &path)))?;
                let mut toml_str = String::new();
                file.read_to_string(&mut toml_str)
                    .or(Err(anyhow!("Cannot read file {}", &path)))?;
                let t: T = toml::from_str(toml_str.as_str())
                    .map_err(|err| anyhow!("Failed to parse {}: {}", &path, err.message()))?;
                Ok((t, path))
            }
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct Import {
    pub import: String,
}

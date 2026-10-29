use std::collections::BTreeMap;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("parse {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone {
    pub number: u8,
    pub name: String,
    pub floor: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub number: u8,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub listen: SocketAddr,
    pub serial: String,
    pub zones: Vec<Zone>,
    pub inputs: Vec<Input>,
}

#[derive(Debug, Deserialize)]
struct ZoneFile {
    name: String,
    floor: u16,
}

#[derive(Debug, Deserialize)]
struct File {
    listen: String,
    serial: String,
    zone: BTreeMap<String, ZoneFile>,
    input: BTreeMap<String, String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let text = fs::read_to_string(path).map_err(|source| Error::Read {
            path: path.display().to_string(),
            source,
        })?;
        let file: File = toml::from_str(&text).map_err(|source| Error::Parse {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_file(file)
    }

    fn from_file(file: File) -> Result<Self, Error> {
        let listen = file.listen.parse::<SocketAddr>().map_err(|_| {
            Error::Invalid(format!("listen address {} is not host:port", file.listen))
        })?;
        if file.serial.is_empty() {
            return Err(Error::Invalid("serial path is empty".into()));
        }
        if file.zone.is_empty() {
            return Err(Error::Invalid("config has no zones".into()));
        }
        if file.input.is_empty() {
            return Err(Error::Invalid("config has no inputs".into()));
        }

        let mut zones = Vec::with_capacity(file.zone.len());
        for (key, spec) in file.zone {
            let number = parse_number("zone", &key, 1, 18)?;
            let name = spec.name.trim();
            if name.is_empty() {
                return Err(Error::Invalid(format!("zone {number} has an empty name")));
            }
            if name.contains('/') {
                return Err(Error::Invalid(format!(
                    "zone {number} name {name} cannot contain /"
                )));
            }
            zones.push(Zone {
                number,
                name: name.to_string(),
                floor: spec.floor,
            });
        }
        zones.sort_by(|a, b| a.floor.cmp(&b.floor).then(a.name.cmp(&b.name)));
        reject_duplicate_names("zone", zones.iter().map(|zone| zone.name.as_str()))?;

        let mut inputs = Vec::with_capacity(file.input.len());
        for (key, label) in file.input {
            let number = parse_number("input", &key, 2, 6)?;
            let name = label.trim();
            if name.is_empty() {
                return Err(Error::Invalid(format!("input {number} has an empty name")));
            }
            if name.contains('/') {
                return Err(Error::Invalid(format!(
                    "input {number} name {name} cannot contain /"
                )));
            }
            inputs.push(Input {
                number,
                name: name.to_string(),
            });
        }
        inputs.sort_by_key(|input| input.number);
        reject_duplicate_names("input", inputs.iter().map(|input| input.name.as_str()))?;

        Ok(Config {
            listen,
            serial: file.serial,
            zones,
            inputs,
        })
    }
}

fn parse_number(kind: &str, key: &str, low: u8, high: u8) -> Result<u8, Error> {
    let number: u16 = key
        .parse()
        .map_err(|_| Error::Invalid(format!("{kind} key {key} is not a number")))?;
    if (low as u16..=high as u16).contains(&number) {
        Ok(number as u8)
    } else {
        Err(Error::Invalid(format!(
            "{kind} {number} is outside {low}–{high}"
        )))
    }
}

fn reject_duplicate_names<'a>(
    kind: &str,
    names: impl IntoIterator<Item = &'a str>,
) -> Result<(), Error> {
    let mut seen = BTreeMap::<&str, ()>::new();
    for name in names {
        if seen.insert(name, ()).is_some() {
            return Err(Error::Invalid(format!(
                "two {kind} entries share the name {name}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Config {
        Config::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("niles.toml")).unwrap()
    }

    #[test]
    fn installed_labels_sort_by_floor_then_name() {
        let config = sample();
        let names: Vec<_> = config.zones.iter().map(|zone| zone.name.as_str()).collect();
        assert_eq!(
            names,
            ["Garage", "Master", "Patio", "Study", "Balcony", "Game"]
        );
        assert_eq!(config.zones[0].number, 3);
        assert_eq!(config.zones[4].floor, 2);
        assert_eq!(
            config
                .inputs
                .iter()
                .map(|input| (input.number, input.name.as_str()))
                .collect::<Vec<_>>(),
            [(3, "Bluetooth"), (4, "TV"), (5, "AirPlay")]
        );
        assert_eq!(config.serial, "/dev/ttyUSB0");
        assert_eq!(config.listen.to_string(), "0.0.0.0:8080");
    }

    #[test]
    fn rejects_tuner_zone_range_and_missing_floor() {
        let base = include_str!("../niles.toml");
        let tuner = base.replace("3 = \"Bluetooth\"", "1 = \"Radio\"");
        assert!(Config::from_file(toml::from_str(&tuner).unwrap()).is_err());

        let zone = base.replace("[zone.6]", "[zone.19]");
        assert!(Config::from_file(toml::from_str(&zone).unwrap()).is_err());

        let floor = base.replace(
            "[zone.2]\nname = \"Study\"\nfloor = 1\n",
            "[zone.2]\nname = \"Study\"\n",
        );
        assert!(toml::from_str::<File>(&floor).is_err());
    }

    #[test]
    fn floor_zero_sorts_before_floor_one() {
        let text = r#"
listen = "127.0.0.1:1"
serial = "/dev/ttyUSB0"

[zone.1]
name = "Attic"
floor = 1

[zone.2]
name = "Lobby"
floor = 0

[input]
3 = "Bluetooth"
"#;
        let config = Config::from_file(toml::from_str(text).unwrap()).unwrap();
        let names: Vec<_> = config.zones.iter().map(|zone| zone.name.as_str()).collect();
        assert_eq!(names, ["Lobby", "Attic"]);
        assert_eq!(config.zones[0].floor, 0);
    }
}

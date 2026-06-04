use std::{
    collections::HashSet,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

use common::{AppResult, error::AppError};
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

use crate::paths::{config_path, keystore_path, storage_path};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub(crate) struct Config {
    #[serde(default = "default_keystore")]
    pub(crate) keystore: PathBuf,
    #[serde(default = "default_storage")]
    pub(crate) storage: PathBuf,
    #[serde(default = "default_port")]
    pub(crate) http_port: u16,
    #[serde(default = "default_p2p_port")]
    pub(crate) p2p_port: u16,
    #[serde(default)]
    pub(crate) nodes: HashSet<String>,
}

fn default_keystore() -> PathBuf {
    keystore_path().unwrap()
}

fn default_storage() -> PathBuf {
    storage_path().unwrap()
}

fn default_port() -> u16 {
    9091
}

fn default_p2p_port() -> u16 {
    5413
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keystore: default_keystore(),
            storage: default_storage(),
            http_port: default_port(),
            p2p_port: default_p2p_port(),
            nodes: HashSet::new(),
        }
    }
}

impl Config {
    pub(crate) fn init() -> AppResult<()> {
        let config = Self::default();
        let config = toml::to_string(&config).map_err(|_| AppError::Decode)?;
        match File::create_new(config_path()?) {
            Ok(mut file) => write!(file, "{}", config)?,
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
            Err(e) => return Err(AppError::IO(e)),
        };
        Ok(())
    }

    pub(crate) fn read(path: &Path) -> AppResult<Self> {
        let value = fs::read_to_string(path)?;
        toml::from_str(&value).map_err(|e| AppError::Decoding(e.to_string()))
    }

    pub(crate) fn get_random_node(&self) -> AppResult<String> {
        let mut rng = rand::rng();
        self.nodes
            .iter()
            .choose(&mut rng)
            .cloned()
            .ok_or(AppError::Network)
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "\nhttp_port: {}\naddress: {}\nstorage: {}\nkeystore: {}\n",
            self.http_port,
            self.p2p_port,
            self.storage.to_string_lossy(),
            self.keystore.to_string_lossy()
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn parse() {
        let value = r#"
        p2p_port = 5413
        nodes = []
        "#;
        let config: Config = toml::from_str(value).unwrap();
        assert_eq!(Config::default(), config);
    }

    #[test]
    fn read() {
        let file = NamedTempFile::new().unwrap();
        let config = Config::read(file.path()).unwrap();
        assert_eq!(Config::default(), config);
    }
}

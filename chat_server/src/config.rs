use std::{fs::File, path::PathBuf};

use serde::{Deserialize, Serialize};

use anyhow::Result;

// app config from app.yml
#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // load config from app.yml by serde_yaml
        // read from /etc/config/app.yml, or ./app.yml or from env CHAT_CONFIG
        let ret = match (
            File::open("/etc/config/app.yml"),
            File::open("app.yml"),
            std::env::var("CHAT_CONFIG"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(file),
            (_, Ok(file), _) => serde_yaml::from_reader(file),
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            (_, _, _) => anyhow::bail!("config file not found"),
        };

        Ok(ret?)
    }
}

#[cfg(test)]
impl AppConfig {
    pub fn load_for_test() -> Result<Self> {
        let file = std::env::current_dir()
            .ok()
            .map(|p| p.parent().unwrap().join("app.yml"))
            .map(File::open)
            .transpose()?
            .unwrap();

        Ok(serde_yaml::from_reader(file)?)
    }
}

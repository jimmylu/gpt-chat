use std::{fs::File, path::PathBuf};

use serde::{Deserialize, Serialize};

use anyhow::Result;

// app config from app.yml
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // CARGO_MANIFEST_DIR 是 Rust 在编译时提供的环境变量
        // 它指向当前 crate 的 Cargo.toml 所在目录
        // 对于 chat_server 这个 crate，它会指向 /path/to/project/chat_server
        let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
        // 使用 dbg! 宏在运行时打印 manifest_dir 的值
        // 输出格式类似: [src/config.rs:10] manifest_dir = "/path/to/project/chat_server"
        dbg!(&manifest_dir);
        // read from /etc/config/chat.yml, or ./chat.yml or from env CHAT_CONFIG
        let ret = match (
            File::open("/etc/config/chat.yml"),
            File::open(manifest_dir.join("chat.yml")),
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

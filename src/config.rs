use config::{Config, ConfigError, File, FileFormat};
use secrecy::Secret;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub application_host: String,
    pub application_port: u16,
    pub postgres: PostgresSettings,
    pub redis: RedisSettings,
    pub task1_email_confirm: Task1Settings,
    pub task1_captcha: Task1Settings,
}

impl Settings {
    pub fn new(file_path: &str) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::new(file_path, FileFormat::Yaml))
            .build()?
            .try_deserialize()
    }
}

#[derive(Clone, Deserialize)]
pub struct PostgresSettings {
    pub url: Secret<String>,
}

#[derive(Clone, Deserialize)]
pub struct RedisSettings {
    pub url: String,
    pub password: Secret<String>,
}

#[derive(Clone, Deserialize)]
pub struct Task1Settings {
    pub expiry_time: u64,
    pub deletion_bulk_count: usize,
}

use config::{self, Config};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_rabbitmq_host")]
    pub rabbitmq_host: String,
    #[serde(default = "default_rabbitmq_port")]
    pub rabbitmq_port: u16,
    pub github_token: String,
    pub slurmrestd_host: String,
    pub slurmrestd_port: u16,
    pub slurmrestd_user: String,
    pub slurmrestd_token: String,
}

fn default_rabbitmq_host() -> String {
    "localhost".to_string()
}

fn default_rabbitmq_port() -> u16 {
    5552
}

impl AppConfig {
    pub fn new() -> Result<Self, lib::errors::AppError> {
        let settings = Config::builder()
            .add_source(config::Environment::with_prefix("GHWEBHOOKS_RMQ_CONSUMER"))
            .build()
            .map_err(|e| lib::errors::AppError::ConfigError(e))?;

        settings
            .try_deserialize::<Self>()
            .map_err(|e| lib::errors::AppError::ConfigError(e))
    }
}

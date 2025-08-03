use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub threshold: u64,
    pub alert_repeat: u64,
    smtp_server: String,
    port: u16,
    tls: bool,
    username: String,
    password: String,
    fromname: String,
    recipient: String,
    pub pins: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct MailConfig {
    pub smtp_server: String,
    pub port: u16,
    pub tls: bool,
    pub username: String,
    pub password: String,
    pub recipient: String,
    pub fromname: String,
}

impl Config {
    pub fn read(path: &str) -> Result<Self> {

        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read file at {}", path))?;
        let cfg: Config = serde_json::from_str(&contents)
            .with_context(|| "Failed to parse JSON in config struct")?;
        Ok(cfg)
    }

    pub fn mail_config(&self) -> MailConfig {
        MailConfig {
            smtp_server: self.smtp_server.clone(),
            port: self.port,
            tls: self.tls,
            username: self.username.clone(),
            password: self.password.clone(),
            recipient: self.recipient.clone(),
            fromname: self.fromname.clone(),
        }
    }
}

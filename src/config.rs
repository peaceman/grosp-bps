use anyhow::Context;
use log::info;
use regex::Regex;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

pub type AppConfig = Arc<Config>;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub consul: Consul,
    pub playlist: Playlist,
    pub http: Http,
}

#[derive(Debug, Deserialize)]
pub struct Consul {
    pub base_url: Url,
    #[serde(with = "humantime_serde")]
    pub update_interval: Duration,
}

#[derive(Debug, Deserialize)]
pub struct Playlist {
    pub upstream_base_url: Url,
    pub segment_signing: SegmentSigning,
    pub jwt_validation: JwtValidation,
}

#[derive(Debug, Deserialize)]
pub struct SegmentSigning {
    pub key: String,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
}

#[derive(Debug, Deserialize)]
pub struct JwtValidation {
    pub secret: String,
    #[serde(with = "serde_regex")]
    pub stream_name_pattern: Regex,
}

#[derive(Debug, Deserialize)]
pub struct Http {
    pub socket: SocketAddr,
}

pub fn load_config() -> anyhow::Result<AppConfig> {
    let config_path = get_config_path()?;
    let file = File::open(&config_path)
        .with_context(|| format!("Failed to open config file {}", &config_path))?;

    Ok(Arc::new(serde_yaml::from_reader(BufReader::new(file))?))
}

fn get_config_path() -> anyhow::Result<String> {
    use std::env;

    env::var("APP_CONFIG").or_else(|e| {
        info!(
            "Missing or invalid APP_CONFIG env var, fallback to config.yml; {:?}",
            e
        );
        Ok("config.yml".to_string())
    })
}

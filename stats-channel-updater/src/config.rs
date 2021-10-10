use serde::Deserialize;
use model::Snowflake;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_counter_url: String,
    pub discord_token: String,
    pub channel_id: Snowflake,
}

impl Config {
    pub fn load() -> Config {
        envy::from_env().expect("failed to load config")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::load()
    }
}

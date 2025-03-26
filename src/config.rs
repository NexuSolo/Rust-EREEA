use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub map: MapConfig,
    pub robots: RobotsConfig,
    pub base: BaseConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MapConfig {
    pub seed: u32,
    pub generation_rate: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RobotsConfig {
    pub explorer: ExplorerConfig,
    pub collector: CollectorConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExplorerConfig {
    pub cost_science: usize,
    pub cost_ore: usize,
    pub cost_energy: usize,
    pub vision_range: usize,
    pub move_delay_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CollectorConfig {
    pub cost_science: usize,
    pub cost_ore: usize,
    pub cost_energy: usize,
    pub move_delay_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BaseConfig {
    pub initial_energy: usize,
    pub initial_ore: usize,
    pub initial_science: usize,
    pub initial_explorers: usize,
    pub initial_collectors: usize,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}

use color_eyre::{eyre::eyre, Result};
use serde_yaml::Value;
use std::fs;

const ALACRITTY_CONFIG_FILE: &str = ".config/alacritty/alacritty.yml";

pub fn get_config_file() -> Result<Value> {
    let config_path = dirs::home_dir()
        .map(|home_dir| home_dir.join(ALACRITTY_CONFIG_FILE))
        .ok_or_else(|| eyre!("unable to find home directory"))?;
    let contents = fs::read(&config_path)?;
    // ensure config file is valid
    let parsed: Value = serde_yaml::from_slice(&contents)
        .map_err(|err| eyre!("Alacritty config file is invalid: {}", &err))?;
    Ok(parsed)
}

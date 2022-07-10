use crate::themes::{Theme, ThemeWrapper};
use color_eyre::{eyre::eyre, Result};
use serde_yaml::{Mapping, Value};
use std::{fs, path::PathBuf};

const ALACRITTY_CONFIG_FILE: &str = ".config/alacritty/alacritty.yml";

fn get_config_file() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home_dir| home_dir.join(ALACRITTY_CONFIG_FILE))
        .ok_or_else(|| eyre!("unable to find home directory"))
}

pub fn get_config() -> Result<Value> {
    let config_path = get_config_file()?;
    let contents = fs::read(&config_path).map_err(|err| {
        eyre!(
            "Unable to read Alacritty config file at {}: {}",
            ALACRITTY_CONFIG_FILE,
            &err
        )
    })?;
    // ensure config file is valid
    let parsed: Value = serde_yaml::from_slice(&contents)
        .map_err(|err| eyre!("Alacritty config file is invalid: {}", &err))?;
    Ok(parsed)
}

pub fn get_config_raw() -> Result<String> {
    let config_path = get_config_file()?;
    let contents = fs::read(&config_path).map_err(|err| {
        eyre!(
            "Unable to read Alacritty config file at {}: {}",
            ALACRITTY_CONFIG_FILE,
            &err
        )
    })?;
    let contents = std::str::from_utf8(&contents)?.to_owned();
    Ok(contents)
}

fn transform_config(config: &Mapping, theme: &Theme) -> Result<Mapping> {
    let mut updated = config.to_owned();
    let theme_val = serde_yaml::to_value(&theme)?;
    updated.insert("colors".into(), theme_val);
    Ok(updated)
}

pub fn update_theme(theme_wrapper: &ThemeWrapper) -> Result<()> {
    let config_path = get_config_file()?;
    let mut alacritty_config_file = get_config()?;
    let alacritty_config = alacritty_config_file
        .as_mapping_mut()
        .ok_or_else(|| eyre!("unable to understand alacritty config file structure"))?;
    let updated = transform_config(alacritty_config, &theme_wrapper.colors)?;
    let updated_ser = serde_yaml::to_string(&updated)?;
    fs::write(&config_path, updated_ser).map_err(|err| {
        eyre!(
            "Unable to write to Alacritty config file at {}: {}",
            ALACRITTY_CONFIG_FILE,
            &err
        )
    })
}

pub fn set_config_str(config: &str) -> Result<()> {
    let config_path = get_config_file()?;
    fs::write(&config_path, config).map_err(|err| {
        eyre!(
            "Unable to write to Alacritty config file at {}: {}",
            ALACRITTY_CONFIG_FILE,
            &err
        )
    })
}

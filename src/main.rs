use color_eyre::{eyre::eyre, Result};

mod alacritty;
mod themes;

fn main() -> Result<()> {
    let _theme_map = themes::get_themes();
    let mut alacritty_config_file = alacritty::get_config_file().unwrap();
    let alacritty_config = alacritty_config_file
        .as_mapping_mut()
        .ok_or_else(|| eyre!("unable to understand alacritty config file structure"))?;
    let maybe_current_colors =
        alacritty_config.get_mut(&serde_yaml::Value::String("colors".into()));
    match &maybe_current_colors {
        Some(current_colors) => println!("current colors: {:#?}", &current_colors),
        None => println!("no current colors found in config"),
    }
    // for (name, _theme) in theme_map.iter() {
    //     println!("theme: {}", name);
    // }
    Ok(())
}

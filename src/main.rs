use color_eyre::{eyre::bail, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::thread;
use themes::ThemeWrapper;

mod alacritty;
mod select;
mod themes;

fn main() -> Result<()> {
    let theme_map = themes::get_themes();
    let (preview_tx, preview_rx): (Sender<Option<ThemeWrapper>>, Receiver<Option<ThemeWrapper>>) =
        unbounded();
    let original_config = alacritty::get_config_raw()?;
    let thread_original_config = original_config.clone();
    let handle = thread::spawn(move || {
        while let Ok(maybe_theme) = preview_rx.recv() {
            match maybe_theme {
                Some(theme) => {
                    let _ = alacritty::update_theme(&theme);
                }
                None => {
                    let _ = alacritty::set_config_str(&thread_original_config);
                }
            }
        }
    });
    let select_result = select::select_theme(&theme_map, Some(preview_tx));
    handle.join().unwrap();
    match select_result {
        Ok(result) => match result {
            Some((theme_name, theme_wrapper)) => {
                eprintln!("theme updated to {}", &theme_name);
                let _ = alacritty::update_theme(&theme_wrapper);
                Ok(())
            }
            None => {
                let _ = alacritty::set_config_str(&original_config);
                Ok(())
            }
        },
        Err(err) => {
            let _ = alacritty::set_config_str(&original_config);
            bail!(err)
        }
    }
}

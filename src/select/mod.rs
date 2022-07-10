use crate::themes::*;
use color_eyre::{eyre::eyre, Result};
use crossbeam_channel::Sender;
use skim::prelude::*;
use std::collections::HashMap;
use tuikit::widget::Size;

struct ThemeItem {
    theme_wrapper: Option<ThemeWrapper>,
    preview_channel: Option<Sender<Option<ThemeWrapper>>>,
}

impl SkimItem for ThemeItem {
    fn text(&self) -> Cow<str> {
        match &self.theme_wrapper {
            Some(theme_wrapper) => match &theme_wrapper.name {
                Some(name) => Cow::Borrowed(name),
                None => Cow::Borrowed("(unknown)"),
            },
            None => Cow::Borrowed("<original>"),
        }
    }

    fn preview(&self, context: PreviewContext) -> ItemPreview {
        // side effect to allow updating alacritty config file in a separate thread
        // to get a real preview effect for whatever text currently happens to be in the terminal
        if let Some(ch) = &self.preview_channel {
            if context.selections.is_empty() {
                // signal to exit theme preview thread
                let _ = ch.send(None);
            } else {
                match &self.theme_wrapper {
                    Some(theme_wrapper) => {
                        // signal to preview this theme
                        let _ = ch.send(Some(theme_wrapper.to_owned()));
                    }
                    None => {
                        // signal to revert to original
                        let _ = ch.send(None);
                    }
                }
            }
        }
        ItemPreview::TextWithPos(
            "".to_string(),
            PreviewPosition {
                h_scroll: Size::Fixed(0),
                h_offset: Size::Fixed(0),
                v_scroll: Size::Fixed(0),
                v_offset: Size::Fixed(0),
            },
        )
    }
}

pub fn select_theme(
    theme_map: &HashMap<String, ThemeWrapper>,
    preview_channel: Option<Sender<Option<ThemeWrapper>>>,
) -> Result<Option<(String, ThemeWrapper)>> {
    let options = SkimOptionsBuilder::default()
        .height(Some("30%"))
        .preview(Some(""))
        .reverse(true)
        .build()
        .map_err(|err| eyre!(err))?;
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
    let mut theme_names: Vec<String> = theme_map.keys().into_iter().map(|s| s.to_owned()).collect();
    theme_names.sort();
    // create original theme item to represent reverted config
    let original_item = Arc::new(ThemeItem {
        theme_wrapper: None,
        preview_channel: preview_channel.as_ref().cloned(),
    });
    let _ = tx_item.send(original_item);
    theme_names.iter().for_each(|name| {
        let theme_wrapper = theme_map.get(name).unwrap().to_owned();
        let item = ThemeItem {
            theme_wrapper: Some(theme_wrapper),
            preview_channel: preview_channel.as_ref().cloned(),
        };
        let _ = tx_item.send(Arc::new(item));
    });
    drop(tx_item); // so that skim could know when to stop waiting for more items.
    let selected_items = Skim::run_with(&options, Some(rx_item))
        .map(|out| match out.final_key {
            Key::Enter => out.selected_items,
            _ => vec![],
        })
        .unwrap_or_default();
    let selected = selected_items.first().map(|i| i.output().into_owned());
    let theme = selected
        .map(|theme_name| {
            theme_map
                .get(&theme_name)
                .map(|theme| (theme_name, theme.to_owned()))
        })
        .unwrap_or_default();
    Ok(theme)
}

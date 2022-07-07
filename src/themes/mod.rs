use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize, Serialize)]
pub struct Primary {
    pub background: String,
    pub foreground: String,
    pub dim_foreground: Option<String>,
    pub bright_foreground: Option<String>,
    pub dim_background: Option<String>,
    pub bright_background: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Colors {
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub magenta: String,
    pub cyan: String,
    pub white: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Cursor {
    pub text: Option<String>,
    pub cursor: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Theme {
    pub primary: Primary,
    pub cursor: Option<Cursor>,
    pub normal: Colors,
    pub bright: Option<Colors>,
    pub dim: Option<Colors>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ThemeWrapper {
    pub name: Option<String>,
    pub colors: Theme,
}

pub fn get_themes() -> HashMap<String, Theme> {
    include!("../../themes_out/themes.rs")
}

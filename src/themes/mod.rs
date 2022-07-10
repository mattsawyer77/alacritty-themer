use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Primary {
    pub background: String,
    pub foreground: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_foreground: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bright_foreground: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bright_background: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cursor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Theme {
    pub primary: Primary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
    pub normal: Colors,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bright: Option<Colors>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim: Option<Colors>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ThemeWrapper {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub colors: Theme,
}

pub fn get_themes() -> HashMap<String, ThemeWrapper> {
    include!("../../themes_out/themes.rs")
}

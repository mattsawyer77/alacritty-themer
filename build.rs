use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    ffi::OsString,
    fs,
};
use uneval::funcs::to_file;

const THEMES_DIR: &str = "themes";
const THEMES_OUT_DIR: &str = "themes_out";

// TODO: consider moving the following theme types to a sub crate to share
// between build.rs and the main application code
#[derive(Clone, Deserialize, Serialize)]
pub struct Primary {
    pub background: String,
    pub foreground: String,
    #[serde(default)]
    pub dim_foreground: Option<String>,
    #[serde(default)]
    pub bright_foreground: Option<String>,
    #[serde(default)]
    pub dim_background: Option<String>,
    #[serde(default)]
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
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = current_dir()?;
    let source_path = cwd.join(THEMES_DIR);
    let dest_path = cwd.join(THEMES_OUT_DIR);
    println!("cargo:rerun-if-changed={}", THEMES_DIR);
    fs::DirBuilder::new().recursive(true).create(&dest_path)?;
    let theme_files: HashSet<OsString> = fs::read_dir(&source_path)?
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let is_file = entry.file_type().unwrap().is_file();
            let filename = entry.file_name().as_os_str().to_str()?.to_owned();
            if is_file && (filename.ends_with(".yaml") || filename.ends_with(".yml")) {
                Some(entry.file_name())
            } else {
                None
            }
        })
        .collect();
    if !theme_files.is_empty() {
        let themes: HashMap<String, ThemeWrapper> = theme_files
            .iter()
            .filter_map(|filename| {
                println!("parsing theme file {:?}...", filename);
                let source_file = source_path.join(filename);
                let contents = &fs::read(&source_file).unwrap();
                // ensure source yaml is valid
                let mut maybe_parsed: Result<ThemeWrapper, serde_yaml::Error> =
                    serde_yaml::from_slice(contents);
                match maybe_parsed {
                    Ok(ref mut parsed) => {
                        let name = source_file
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .replace(".yml", "");
                        parsed.name = Some(name.to_owned());
                        let theme_wrapper = parsed.to_owned();
                        Some((name, theme_wrapper))
                    }
                    Err(err) => {
                        eprintln!(
                            "WARNING: could not parse file {}: {}",
                            &filename.to_str().unwrap(),
                            &err
                        );
                        None
                    }
                }
            })
            .collect();
        // write the map of name-theme values as serialized rust to themes_out/themes.rs
        to_file(&themes, dest_path.join("themes.rs")).unwrap();
    } else {
        panic!("no themes found in {}", THEMES_DIR)
    }
    Ok(())
}

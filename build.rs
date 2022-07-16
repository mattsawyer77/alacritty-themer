use markdown::{Block::*, Span::*};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    ffi::OsString,
    fs,
    path::PathBuf,
};
use uneval::funcs::to_file;

const THEMES_DIR: &str = "themes";
const THEMES_OUT_DIR: &str = "themes_out";
const THEMES_WIKI_URL: &str = "https://github.com/alacritty/alacritty.wiki.git";
const THEMES_WIKI_FILENAME: &str = "Color-schemes.md";

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub colors: Theme,
}

fn get_theme_files() -> Result<HashSet<OsString>, Box<dyn std::error::Error>> {
    let cwd = current_dir()?;
    let source_path = cwd.join(THEMES_DIR);
    let dest_path = cwd.join(THEMES_OUT_DIR);
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
    Ok(theme_files)
}

fn get_theme_map(
    theme_files: HashSet<OsString>,
) -> Result<HashMap<String, ThemeWrapper>, Box<dyn std::error::Error>> {
    let cwd = current_dir()?;
    let source_path = cwd.join(THEMES_DIR);
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
    Ok(themes)
}

fn parse_theme_wiki_text(text: &str) -> Option<String> {
    let text = text.trim_start_matches("<details>");
    let selector = Selector::parse(r#"summary"#).unwrap();
    let fragment = Html::parse_fragment(text);
    let summary = fragment.select(&selector).next()?;
    let theme_name = summary.text().next()?.to_owned();
    Some(theme_name)
}

fn parse_theme_wiki_markdown(
    contents: &str,
) -> Result<HashMap<String, ThemeWrapper>, Box<dyn std::error::Error>> {
    let parsed = markdown::tokenize(contents);
    let mut themes: HashMap<String, ThemeWrapper> = HashMap::new();
    if parsed.len() < 2 {
        panic!("failed to parse wiki markdown")
    } else {
        // dbg!(parsed);
        for pairs in parsed.windows(2) {
            if let (Paragraph(spans), CodeBlock(maybe_lang, code_block)) = (&pairs[0], &pairs[1]) {
                if let Some(lang) = maybe_lang {
                    if lang == "yaml" {
                        if let Some(theme_name) = spans.iter().find_map(|span| {
                            if let Text(text) = span {
                                parse_theme_wiki_text(text)
                            } else {
                                None
                            }
                        }) {
                            let maybe_parsed: Result<ThemeWrapper, serde_yaml::Error> =
                                serde_yaml::from_slice(code_block.as_bytes());
                            match maybe_parsed {
                                Ok(parsed) => {
                                    let _ = themes.insert(
                                        theme_name.to_owned(),
                                        ThemeWrapper {
                                            name: Some(theme_name),
                                            colors: parsed.colors.to_owned(),
                                        },
                                    );
                                }
                                Err(err) => println!("could not parse span: {}", &err.to_string()),
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(themes)
}

fn retrieve_themes() -> Result<HashMap<String, ThemeWrapper>, Box<dyn std::error::Error>> {
    // clone alacritty theme wiki into a temp dir, then parse its markdown into themes
    let mut opts = run_script::ScriptOptions::new();
    opts.print_commands = true;
    opts.exit_on_error = true;
    println!("retrieving themes from {}...", THEMES_WIKI_URL);
    let app_config_dir = dirs::config_dir().unwrap().join("alacritty-themer");
    fs::create_dir_all(&app_config_dir)?;
    opts.working_directory = Some(app_config_dir.to_owned());
    let wiki_dir = app_config_dir.join("alacritty.wiki");

    println!("config directory created: '{:?}'", &app_config_dir.to_str(),);
    let empty_args: &Vec<String> = &Vec::new();
    match fs::read_dir(&wiki_dir) {
        Ok(_) => {
            opts = opts.clone();
            opts.working_directory = Some(wiki_dir.to_owned());
            opts.print_commands = true;
            println!(
                "pulling latest alacritty wiki from {} into {}...",
                THEMES_WIKI_URL,
                &wiki_dir.to_string_lossy()
            );
            let (code, output, err) = run_script::run(r#"git pull origin"#, empty_args, &opts)?;
            if code != 0 {
                panic!(
                    "failed to pull latest alacritty wiki,\noutput: {}\nerr: {}",
                    &output, &err
                );
            }
            println!("alacritty wiki updated, output: {}", &output);
        }
        Err(_) => {
            println!("cloning alacritty wiki from {}...", THEMES_WIKI_URL);
            let clone_cmd = vec!["git", "clone", "--depth", "1", THEMES_WIKI_URL].join(" ");
            let (code, output, err) = run_script::run(&clone_cmd, empty_args, &opts)?;
            if code != 0 {
                panic!(
                    "failed to clone alacritty wiki,\noutput: {}\nerr: {}",
                    &output, &err
                );
            }
            println!("alacritty wiki cloned, output: {}", &output);
        }
    }

    let wiki_files = fs::read_dir(&wiki_dir).unwrap();
    wiki_files.for_each(|f| {
        let f = f.unwrap();
        println!(" - {}", &f.file_name().to_string_lossy());
    });
    let wiki_path = wiki_dir.join(THEMES_WIKI_FILENAME);
    println!("reading alacritty wiki file: {:?}", &wiki_path);
    let contents = &fs::read(&wiki_path)?;
    let contents = String::from_utf8_lossy(contents);
    let themes = parse_theme_wiki_markdown(&contents)?;
    Ok(themes)
}

fn update_theme_files(
    theme_map: HashMap<String, ThemeWrapper>,
    out_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    theme_map.iter().for_each(|(theme_name, theme)| {
        println!("serializing theme {}...", theme_name);
        let contents = serde_yaml::to_string(&theme).unwrap();
        let filename = format!("{}.yml", &theme_name);
        let out_file = out_dir.join(&filename);
        println!(
            "writing parsed theme {} to {}",
            theme_name,
            &out_file.to_string_lossy()
        );
        fs::write(out_file, &contents).unwrap();
    });
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", THEMES_DIR);
    let cwd = current_dir()?;
    let dest_path = cwd.join(THEMES_OUT_DIR);
    if let Ok(should_retrieve_themes) = std::env::var("RETRIEVE_THEMES") {
        if should_retrieve_themes.eq_ignore_ascii_case("true") {
            let themes = retrieve_themes()?;
            update_theme_files(themes, PathBuf::from(THEMES_DIR))?;
        }
    }
    let theme_files = get_theme_files()?;
    if !theme_files.is_empty() {
        let themes = get_theme_map(theme_files)?;
        // write the map of name-theme values as serialized rust to themes_out/themes.rs
        to_file(&themes, dest_path.join("themes.rs")).unwrap();
    } else {
        panic!("no themes found in {}", THEMES_DIR)
    }
    Ok(())
}

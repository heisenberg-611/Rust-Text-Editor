use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Config {
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct EditorConfig {
    #[serde(default = "default_line_numbers")]
    pub line_numbers: bool,
    #[serde(default = "default_mouse_support")]
    pub mouse_support: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ThemeConfig {
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default = "default_foreground")]
    pub foreground: String,
    #[serde(default = "default_cursor")]
    pub cursor: String,
    #[serde(default = "default_selection_bg")]
    pub selection_bg: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig {
                line_numbers: true,
                mouse_support: true,
            },
            theme: ThemeConfig {
                background: "#1e1e1e".into(),
                foreground: "#ffffff".into(),
                cursor: "#cccccc".into(),
                selection_bg: "#3e4451".into(),
            },
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            line_numbers: true,
            mouse_support: true,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: "#1e1e1e".into(),
            foreground: "#ffffff".into(),
            cursor: "#cccccc".into(),
            selection_bg: "#3e4451".into(),
        }
    }
}

fn default_line_numbers() -> bool { true }
fn default_mouse_support() -> bool { true }
fn default_background() -> String { "#1e1e1e".to_string() }
fn default_foreground() -> String { "#ffffff".to_string() }
fn default_cursor() -> String { "#cccccc".to_string() }
fn default_selection_bg() -> String { "#3e4451".to_string() }

impl Config {
    pub fn load() -> Self {
        // Try to load from .config/config.toml
        let config_path = Path::new(".config/config.toml");
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(config_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return config;
                }
            }
        }
        Config::default()
    }
}

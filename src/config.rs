use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub editor: EditorConfig,
    pub theme: ThemeConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct EditorConfig {
    #[serde(default = "default_tab_size")]
    pub tab_size: usize,
    #[serde(default = "default_line_numbers")]
    pub line_numbers: bool,
    #[serde(default = "default_mouse_support")]
    pub mouse_support: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
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
    #[serde(default = "default_status_bg")]
    pub status_bg: String,
    #[serde(default = "default_status_fg")]
    pub status_fg: String,
    // Syntax highlighting
    #[serde(default = "default_keyword")]
    pub keyword: String,
    #[serde(default = "default_string")]
    pub string: String,
    #[serde(default = "default_comment")]
    pub comment: String,
    #[serde(default = "default_number")]
    pub number: String,
    #[serde(default = "default_type_color")]
    pub type_color: String,
    #[serde(default = "default_control_flow")]
    pub control_flow: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            theme: ThemeConfig::default(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            line_numbers: true,
            mouse_support: true,
            theme: "default".into(),
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
            status_bg: "#3e4451".into(),
            status_fg: "#ffffff".into(),
            keyword: "#c678dd".into(),
            string: "#98c379".into(),
            comment: "#5c6370".into(),
            number: "#d19a66".into(),
            type_color: "#e5c07b".into(),
            control_flow: "#c678dd".into(),
        }
    }
}

fn default_tab_size() -> usize {
    4
}
fn default_line_numbers() -> bool {
    true
}
fn default_mouse_support() -> bool {
    true
}
fn default_theme() -> String {
    "default".to_string()
}
fn default_background() -> String {
    "#1e1e1e".to_string()
}
fn default_foreground() -> String {
    "#ffffff".to_string()
}
fn default_cursor() -> String {
    "#cccccc".to_string()
}
fn default_selection_bg() -> String {
    "#3e4451".to_string()
}
fn default_status_bg() -> String {
    "#3e4451".to_string()
}
fn default_status_fg() -> String {
    "#ffffff".to_string()
}
fn default_keyword() -> String {
    "#c678dd".to_string()
}
fn default_string() -> String {
    "#98c379".to_string()
}
fn default_comment() -> String {
    "#5c6370".to_string()
}
fn default_number() -> String {
    "#d19a66".to_string()
}
fn default_type_color() -> String {
    "#e5c07b".to_string()
}
fn default_control_flow() -> String {
    "#c678dd".to_string()
}

impl Config {
    pub fn load() -> Self {
        // Try to load from .config/config.toml
        let config_path = Path::new(".config/config.toml");
        let mut config = if config_path.exists() {
            if let Ok(content) = fs::read_to_string(config_path) {
                toml::from_str(&content).unwrap_or_else(|_| Config::default())
            } else {
                Config::default()
            }
        } else {
            Config::default()
        };

        // If a specific theme is requested, load it
        if config.editor.theme != "default" {
            config.theme = crate::theme::load_theme(&config.editor.theme);
        }

        config
    }
}

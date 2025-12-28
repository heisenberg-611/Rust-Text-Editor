use crate::config::ThemeConfig;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_theme(name: &str) -> ThemeConfig {
    if name == "default" {
        return ThemeConfig::default();
    }

    let mut theme_path = PathBuf::from(format!(".config/themes/{}.toml", name));

    if let Ok(home) = env::var("HOME") {
        let global_path = Path::new(&home).join(format!(".config/meow/themes/{}.toml", name));
        if global_path.exists() {
            theme_path = global_path;
        }
    }

    let path = Path::new(&theme_path);

    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(theme) = toml::from_str::<ThemeConfig>(&content) {
                return theme;
            }
        }
    }

    ThemeConfig::default()
}

use crate::config::ThemeConfig;
use std::fs;
use std::path::Path;

pub fn load_theme(name: &str) -> ThemeConfig {
    if name == "default" {
        return ThemeConfig::default();
    }

    let theme_path = format!(".config/themes/{}.toml", name);
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

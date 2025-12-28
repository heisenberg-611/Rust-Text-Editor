mod config;
mod document;
mod editor;
mod row;
mod syntax;
mod terminal;
mod theme;

use config::Config;
use editor::Editor;

fn main() {
    let config = Config::load();
    let mut editor = Editor::new(config);
    editor.run();
}

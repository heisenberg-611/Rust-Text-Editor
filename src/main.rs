mod config;
mod document;
mod editor;
mod row;
mod syntax;
mod terminal;
mod theme;

use editor::Editor;

fn main() {
    let mut editor = Editor::new();
    editor.run();
}

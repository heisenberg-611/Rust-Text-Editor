mod config;
mod document;
mod editor;
mod row;
mod terminal;

use editor::Editor;

fn main() {
    let mut editor = Editor::new();
    editor.run();
}

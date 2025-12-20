# Text Editor in Rust

A modal text editor inspired by Neovim, built with Rust.

## Features

- **Modal Editing**: Normal, Insert, Command, and **Visual** modes.
- **Vim-like Keybindings**: `hjkl` navigation, `i` for insert, `v` for visual, `:w`, `:q` commands.
- **Clipboard Integration**: Copy (`y`) and Paste (`p`) using system clipboard.
- **Mouse Support**: Click to move cursor.
- **Configuration**: Customizable via `.config/config.toml`.
- **Performance**: Built with `ratatui` and `crossterm` for high-performance TUI.

## Tech Stack

- **Language**: Rust
- **Rendering**: [ratatui](https://github.com/ratatui/ratatui) - A Rust TUI library.
- **Input/Terminal**: [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation.
- **Configuration**: [serde](https://serde.rs/) & [toml](https://toml.io) - For parsing the config file.

## How to Run

1. **Prerequisites**: Ensure you have Rust installed (`cargo`).
2. **Build**:
   ```bash
   cargo build --release
   ```
3. **Install**:
   ```bash
   cargo install --path .
   ```
   This will install the `meow` binary to your `~/.cargo/bin` folder.
4. **Run**:
   ```bash
   meow filename.txt
   ```

## Customization

The editor looks for a configuration file at `.config/config.toml` (relative to the working directory).

### Default Configuration

```toml
[editor]
line_numbers = true
mouse_support = true

[theme]
background = "#1e1e1e"
foreground = "#ffffff"
cursor = "#cccccc"
selection_bg = "#3e4451"
```

## Keybindings

### Normal Mode
- `h`, `j`, `k`, `l`: Move cursor Left, Down, Up, Right.
- `i`: Enter Insert Mode.
- `v`: Enter Visual Mode.
- `x`: Delete character under cursor.
- `p`: Paste from system clipboard.
- `:`; Enter Command Mode.

### Insert Mode
- `Esc`: Return to Normal Mode.
- Type normal characters to insert them.

### Visual Mode
- Move cursor to select text.
- `y`: Yank (Copy) selection to system clipboard.
- `d` / `x`: Cut selection to system clipboard.
- `Esc`: Return to Normal Mode.

### Command Mode
- `:w` or `:w <filename>`: Save file.
- `:q`: Quit (unsaved changes will be lost immediately in this version).
- `:wq`: Save and Quit.

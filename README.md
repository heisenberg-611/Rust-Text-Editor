# Text Editor in Rust

<div align="center">

[![Apache 2.0 License](https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=for-the-badge)](https://github.com/heisenberg-611/Rust-Text-Editor/blob/master/LICENSE.txt)
[![GitHub stars](https://img.shields.io/github/stars/heisenberg-611/Rust-Text-Editor?style=for-the-badge&color=gold)](https://github.com/heisenberg-611/Rust-Text-Editor/stargazers)
[![Rust](https://img.shields.io/badge/language-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Repo Size](https://img.shields.io/github/repo-size/heisenberg-611/Rust-Text-Editor?style=for-the-badge&color=brightgreen)](https://github.com/heisenberg-611/Rust-Text-Editor)

[![LinkedIn](https://img.shields.io/badge/LinkedIn-Profile-blue?style=for-the-badge&logo=linkedin)](https://www.linkedin.com/in/dhrubojyoti-saha-b75639349/)
[![Twitter/X](https://img.shields.io/badge/Twitter-X-black?style=for-the-badge&logo=x)](https://twitter.com/Dhrubojyoti279)

</div>

A modal text editor inspired by Neovim, built with Rust.

![Screenshot of editor](https://i.imgur.com/WVURAPQ.jpeg)
## Features

- **Modal Editing**: Normal, Insert, Command, and **Visual** modes.
- **Vim-like Keybindings**: `hjkl` navigation, `i` for insert, `v` for visual, `:w`, `:q` commands.
- **Clipboard Integration**: Copy (`y`) and Paste (`p`) using system clipboard.
- **Mouse Support**: Click to move cursor.
- **Extensive Themes**: 12+ built-in themes (Dracula, Nord, Catppuccin, etc.) with external file support.
- **Syntax Highlighting**: Auto-detection for Rust, C, C++, Java, JS/TS, and Python. Distinguishes between keywords, types, and control flow.
- **Basic Auto-Completion**: Context-aware suggestions while typing.
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
   
   **Option A: Automatic Installation (Recommended)**
   Run the installation script to build, install, and configure your PATH automatically:
   ```bash
   chmod +x install.sh
   ./install.sh
   ```

   **Option B: Manual Installation**
   ```bash
   cargo install --path .
   ```
   Add the binary path to your shell configuration (e.g., `.zshrc`, `.bashrc`):
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

4. **Run**:
   ```bash
   meow filename.txt
   ```

## Customization

The editor looks for a configuration file at `~/.config/meow/config.toml` (global) or `.config/config.toml` (local override).

### Default Configuration

```toml
[editor]
line_numbers = true
mouse_support = true
theme = "dracula" # Set your preferred theme here

[theme]
background = "#1e1e1e"
foreground = "#ffffff"
cursor = "#cccccc"
selection_bg = "#3e4451"
```

## Keybindings

### Normal Mode
- `h`, `j`, `k`, `l` or **Arrow Keys**: Move cursor Left, Down, Up, Right.
- `i`: Enter Insert Mode.
- `v`: Enter Visual Mode.
- `x`: Delete character under cursor.
- `p`: Paste from system clipboard.
- `/`: Enter Search Mode.
- `n` / `N`: Next / Previous search match.
- `:`; Enter Command Mode.
- Scrolling function

### Insert Mode
- `Esc`: Return to Normal Mode.
- **Arrow Keys**: Navigate while typing.
- **Backspace**: Delete characters (merges lines if at start).
- **Auto-Completion**:
    - Typing triggers suggestions.
    - `Up` / `Down`: Navigate suggestions.
    - `Tab` / `Enter`: Select suggestion.
- Type normal characters to insert them.

### Visual Mode
- Move cursor or **Arrow Keys** to select text.
- `y`: Yank (Copy) selection to system clipboard.
- `d`: Delete selection.
- `x`: Cut selection (Copy to clipboard and delete).
- `Esc`: Return to Normal Mode.

### Command Mode
- `:w` or `:w <filename>`: Save file.
- `:q`: Quit (unsaved changes will be lost immediately in this version).
- `:wq`: Save and Quit.

Open [DOCUMENTATION.md](DOCUMENTATION.md) for user docs, or [STUDY_GUIDE.md](STUDY_GUIDE.md) for a deep dive architectural manual.

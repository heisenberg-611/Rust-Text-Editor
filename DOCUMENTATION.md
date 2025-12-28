# Meow Text Editor - Comprehensive Documentation

**Meow** is a lightweight, modal text editor built in Rust. Inspired by Vim, it offers a keyboard-centric code editing experience with a modern touch, including custom themes, mouse support, basic auto-completion, and system clipboard integration.

---

## 1. Installation

### Prerequisites
You must have **Rust** and **Cargo** installed.
-   **Install Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Installing `meow`

**Option 1: Automatic Installation (Recommended)**
Run the included installation script to build, install, and configure your PATH automatically:

```bash
chmod +x install.sh
./install.sh
```

**Option 2: Manual Installation**
Run the following command in the project root to install `meow` to your cargo bin directory:

```bash
cargo install --path .
```

*Note: You must ensure `~/.cargo/bin` is in your system's PATH if you choose manual installation.*

### Platforms
-   **macOS**: Supported & Tested.
-   **Linux**: Supported (Requires `libxcb` for clipboard: `sudo apt-get install libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`).
-   **Windows**: Supported (Use PowerShell or CMD).

---

## 2. User Guide

### Running the Editor
```bash
meow <filename>
# Example: meow main.rs
```

### Modes
Meow is a **modal** editor. This means keys behave differently depending on the active mode.

| Mode | Indicator | Description | How to Enter | How to Exit |
| :--- | :--- | :--- | :--- | :--- |
| **Normal** | `NORMAL` | Navigation & commands (Default). | `Esc` | - |
| **Insert** | `INSERT` | Typing text. | `i` | `Esc` |
| **Visual** | `VISUAL` | Selecting text. | `v` or Mouse Drag | `Esc` |
| **Command**| `COMMAND`| Saving/Quitting (`:w`, `:q`). | `:` | `Enter` or `Esc` |
| **Search** | `SEARCH` | Finding text. | `/` | `Enter` or `Esc` |
| **Auto-Complete** | (Popup) | Selecting suggestions. | Typing | `Esc` (dismiss), `Enter` (select) |

### Auto-Completion
When in **Insert Mode**, typing characters triggers a popup list of suggestions (based on keywords, types, and other text in the file).
- **Navigation**: Use `Up` / `Down` arrows to cycle through suggestions.
- **Selection**: Press `Tab` or `Enter` to insert the selected suggestion.
- **Dismiss**: Press `Esc` to close the popup without selecting.

### Keybindings Cheat Sheet

#### Navigation (Normal Mode)
-   `h`, `j`, `k`, `l` or **Arrow Keys**: Left, Down, Up, Right
-   `Mouse Click`: Move cursor to position.

#### Editing
-   `i`: Enter Insert Mode.
-   `x`: Delete character at cursor.
-   `p`: Paste from clipboard.
-   `Backspace`: Deletes characters. If cursor is at the start of a line, it merges the line with the previous one.
-   **Auto-Complete**:
    -   `Up` / `Down`: Navigate suggestions popup.
    -   `Tab` / `Enter`: Insert selected suggestion.

#### Visual Mode (Selection)
-   `v`: Start Visual Mode.
-   `h`/`j`/`k`/`l` or **Arrow Keys**: Extend selection.
-   `y`: **Yank** (Copy) selected text to clipboard.
-   `d`: **Delete** selected text (no clipboard).
-   `x`: **Cut** selected text to clipboard.

#### Search
-   `/`: Enter Search Mode.
-   Type query and press `Enter`.
-   `n`: Jump to **next** match.
-   `N`: Jump to **previous** match.

#### Commands
-   `:w`: Save file.
-   `:q`: Quit.
-   `:wq`: Save and Quit.

---

## 3. Configuration
Meow looks for a configuration file at `~/.config/meow/config.toml` (recommended) or `.config/config.toml` in the current working directory.
To set this up, run:
```bash
cp config.toml ~/.config/meow/config.toml
```
Themes are loaded from `~/.config/meow/themes/` or locally from `.config/themes/`.

### Example `config.toml`
```toml
[editor]
tab_size = 4
line_numbers = true
mouse_support = true
theme = "dracula" # See available themes below

[theme]
# (Optional) Overrides for specific colors if "theme" is set to "default" or left empty.
background = "#282c34"
foreground = "#abb2bf"
cursor = "#528bff"
selection_bg = "#3e4451"
```

### Available Themes
You can set `theme` in `config.toml` to one of the following:
- `default` (uses colors defined in `[theme]` section)
- `dracula`
- `nord`
- `catppuccin_mocha`
- `catppuccin_latte`
- `gruvbox_dark`
- `gruvbox_light`
- `onedark`
- `monokai`
- `solarized_dark`
- `solarized_light`
- `tokyonight`
- `ayu_mirage`

Themes are loaded from `.config/themes/` if they exist as TOML files.

### Status Bar Info
-   **Mode Indicator**: Shows current editor mode.
-   **File Info**: Displays filename, line count, and **total byte size**.
-   **Save Feedback**: On `:w`, displays the number of bytes written.
-   **Dynamic Status**: Messages like "Yanked!" appear temporarily and clear after 5 seconds or upon next action.

---

## 4. Architecture Guide (For Developers)

The codebase is organized into modular components.

### File Structure
-   **`src/main.rs`**: Entry point. initializes the `Editor` and starts the event loop.
-   **`src/editor.rs`**: The brain of the application.
    -   `Editor` struct: Holds state (cursor pos, document, mode, config).
    -   `process_keypress`: Handles inputs based on active Mode.
    -   `refresh_screen`: Orchestrates rendering to the TUI.
-   **`src/document.rs`**: Manages the text buffer.
    -   `Document` struct: Manages text using a `Rope` (via `ropey` crate) for efficient editing.
    -   `find`: Implements the search logic (forward/backward with wrap-around).
    -   `insert`/`delete`: Low-level text manipulation.
-   **`src/row.rs`**: Represents a single line of text.
    -   `render`: Handles rendering math (scrolling) and Unicode safety.
-   **`src/terminal.rs`**: Interface with `crossterm`.
    -   Handles raw mode, screen clearing, and low-level I/O.
-   **`src/config.rs`**: Uses `serde` to parse `config.toml`.

### Data Flow
1.  **Input**: User presses a key → `crossterm` catches it.
2.  **Process**: `Editor::process_keypress` matches the key against the current `Mode`.
3.  **Update**: `Document` is modified, or `Editor` state (cursor, mode) changes.
4.  **Render**: `Editor::refresh_screen` redraws the interface using `ratatui`.

---

## 5. Extending the Editor

This section describes how to add support for new features.

### Adding Syntax Highlighting for a New Language

1.  Open `src/syntax.rs`.
2.  Add a new entry to the `SYNTAX_LIST` array.
3.  Define the following properties:
    ```rust
    Syntax {
        file_type: "YourLanguage",
        file_extensions: &["ext"], // e.g. ["rb", "ruby"]
        keywords: &[
             "keyword1", "keyword2" // Add language keywords here
        ],
        single_line_comment: "#", // The comment starter string
    },
    ```
4.  Rebuild the editor: `cargo install --path .`

---

## 6. Troubleshooting & FAQ

### "Command not found: meow"
-   **Cause**: `~/.cargo/bin` is not in your PATH.
-   **Fix**: Run `./install.sh` to fix this automatically, or manually add `export PATH="$HOME/.cargo/bin:$PATH"` to your shell profile.

### Text Rendering Weirdness
-   **Cause**: Multibyte characters (emojis) or tab widths.
-   **Fix**: We use `unicode-width` crate to handle widths, but simple terminals might struggle. Try a modern terminal like Alacritty, iTerm2, or Windows Terminal.

### Clipboard Not Working (Linux)
-   **Cause**: Missing X11 dev libraries.
-   **Fix**: Install `libxcb` dependencies (see Installation section).

### How to Uninstall
```bash
cargo uninstall Rust-text-editor
```
（If installed via `cargo install --path .`, the package name defaults to the one in `Cargo.toml`).

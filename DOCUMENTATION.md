# Meow Text Editor - Comprehensive Documentation

**Meow** is a lightweight, modal text editor built in Rust. Inspired by Vim, it offers a keyboard-centric code editing experience with a modern touch, including custom themes, mouse support, and system clipboard integration.

---

## 1. Installation

### Prerequisites
You must have **Rust** and **Cargo** installed.
-   **Install Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Installing `meow`
Run the following command in the project root to install `meow` to your cargo bin directory:

```bash
cargo install --path .
```

*Note: Ensure `~/.cargo/bin` is in your system's PATH.*

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

### Keybindings Cheat Sheet

#### Navigation (Normal Mode)
-   `h`, `j`, `k`, `l` or **Arrow Keys**: Left, Down, Up, Right
-   `Mouse Click`: Move cursor to position.

#### Editing
-   `i`: Enter Insert Mode.
-   `x`: Delete character at cursor.
-   `p`: Paste from clipboard.
-   `Backspace`: Deletes characters. If cursor is at the start of a line, it merges the line with the previous one.

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

Meow looks for a configuration file at `.config/config.toml` in the project directory (or working directory).

### Example `config.toml`
```toml
[editor]
tab_size = 4
line_numbers = true
mouse_support = true

[theme]
# Hex color codes (RRGGBB)
background = "#282c34"   # Main editor background
foreground = "#abb2bf"   # Default text color
cursor = "#528bff"       # Cursor color
selection_bg = "#3e4451" # Background color for selected text
```

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
    -   `Document` struct: A vector of `Row`s.
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

## 5. Troubleshooting & FAQ

### "Command not found: meow"
-   **Cause**: `~/.cargo/bin` is not in your PATH.
-   **Fix**: Run `export PATH="$HOME/.cargo/bin:$PATH"` or add it to your shell profile (`.zshrc` / `.bashrc`).

### Text Rendering Weirdness
-   **Cause**: Multibyte characters (emojis) or tab widths.
-   **Fix**: We use `unicode-width` crate to handle widths, but simple terminals might struggle. Try a modern terminal like Alacritty, iTerm2, or Windows Terminal.

### Clipboard Not Working (Linux)
-   **Cause**: Missing X11 dev libraries.
-   **Fix**: Install `libxcb` dependencies (see Installation section).

### How to Uninstall
```bash
cargo uninstall text-editor
```
（If installed via `cargo install --path .`, the package name defaults to the one in `Cargo.toml`).

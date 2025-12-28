# Meow Text Editor - The Complete Study Guide

**Version**: 3.1416.0
**Author**: Dhrubojyoti Saha (heisenberg-611)

---

## 1. Introduction

### What is Meow?
Meow is a modal text editor built in Rust. It draws heavy inspiration from Vim/Neovim, focusing on keyboard-centric efficiency. Unlike standard "modeless" editors (like Notepad or VS Code) where you are always typing text, Meow operates in distinct **modes** (Normal, Insert, Visual, etc.). This separation allows keys to perform different actions depending on the context, maximizing productivity without needing a mouse.

### Why Rust?
Rust was chosen for this project for three key reasons:
1.  **Performance**: Rust compiles to native code and rivals C/C++ in speed, essential for a low-latency text editor.
2.  **Memory Safety**: Rust's ownership model prevents common crashes (segfaults, null pointer exceptions) without needing a garbage collector.
3.  **Ecosystem**: Libraries (crates) like `ratatui` (TUI), `crossterm` (Terminal I/O), and `serde` (Configuration) drastically rapidly accelerate development.

---

## 2. Architecture Overview

Meow follows a **Model-View-Controller (MVC)**-like pattern, though simplified for a terminal application.

### The Big Picture
1.  **Event Loop**: The program runs in an infinite loop (`Editor::run`).
2.  **Input (Controller)**: It waits for user input (keyboard/mouse) using `crossterm`.
3.  **Process (Model)**: The input modifies the editor state (cursor position, text buffer, active mode).
4.  **Render (View)**: The current state is drawn to the terminal screen using `ratatui`.

### Directory Structure
*   `src/main.rs`: The entry point. Initializes configuration and the Editor.
*   `src/editor.rs`: The core logic. Handles the event loop, key processing, and drawing commands.
*   `src/document.rs`: Handles the data. load, save, insert, delete operations on the text.
*   `src/row.rs`: Represents a single line of text. Handles visual rendering (tabs, width).
*   `src/terminal.rs`: Wrapper around `crossterm` and `ratatui` to manage the raw terminal interface.
*   `src/config.rs`: Loads user settings from `config.toml` (global or local).
*   `src/theme.rs`: Loads color schemes.
*   `src/syntax.rs`: Logic for syntax highlighting (keywords, types, comments).

---

## 3. Core Components Deep Dive

### 3.1. Main Entry (`main.rs`)
The program starts here. It:
1.  Loads configuration (`Config::load()`).
2.  Creates an `Editor` instance.
3.  Calls `editor.run()`, which blocks until the user quits.

### 3.2. The Editor Engine (`editor.rs`)
This is the "brain" of the application.

*   **State**: Holds the `Document` (text), `CursorPosition`, `Mode`, and `Terminal` interface.
*   **Modes**:
    *   `Normal`: Default mode. Keys are commands (e.g., `h`, `j` move cursor, `i` switches to Insert).
    *   `Insert`: Keys are typed into the document. `Esc` goes back to Normal.
    *   `Visual`: For selecting text. `v` starts it.
    *   `Command`: For editor commands (`:w`, `:q`).
    *   `Search`: For finding text (`/`).
*   **The Loop**:
    ```rust
    pub fn run(&mut self) {
        loop {
            self.refresh_screen(); // Draw UI
            if self.should_quit { break; }
            self.process_keypress(); // Handle Input
        }
    }
    ```

### 3.3. Text Management (`document.rs` & `row.rs`)
Meow uses a **Rope** data structure (specifically the `ropey` crate) to manage text efficiently.
*   **Rope**: A tree-based data structure that allows for O(log N) insertions and deletions, making it much faster than a standard String or Vector of lines for large files.
*   **Row**: For rendering, the editor slices the Rope into lines.
*   **Operations**:
    *   `insert(position, char)`: Inserts a character into the Rope at the calculated byte index.
    *   `delete(position)`: Removes a character or range from the Rope. The Rope handles line merging automatically.

### 3.4. The View Layer (`terminal.rs` & `ratatui`)
Terminals usually line-buffer input (wait for Enter). Meow uses **Raw Mode**:
*   **Raw Mode**: Disables line buffering and echo. Every keystroke is sent immediately to the program.
*   **Alternate Screen**: Switches to a secondary buffer so the terminal history isn't messed up when Meow exits.
*   **Rendering**: `refresh_screen()` calculates what lines are visible (scrolling) and draws them. It uses `ratatui` widgets (Paragraphs) to render text with styles (colors).

### 3.5. Configuration & Themes
*   **Global Config**: Checks `~/.config/meow/config.toml`.
    *   Setup: `cp config.toml ~/.config/meow/config.toml`
*   **Themes**: TOML files defining color palettes. The editor parses hex codes (e.g., `#ff0000`) and converts them to `Rgb` colors for the terminal.

---

## 4. Key Concepts & Algorithms

### Scrolling
The editor maintains an `offset` (x, y).
*   **Rendering**: Only draws rows automatically starting from `offset.y` and chars from `offset.x`.
*   **Cursor Tracking**: If the cursor moves off-screen, the offset is updated to "scroll" the view to keep the cursor visible.

### Syntax Highlighting (`syntax.rs`)
A simple but effective highlighter:
1.  Iterates through characters in a Row.
2.  Matches against known **Keywords** (e.g., `fn`, `let`), **Types**, and **Comments**.
3.  Assigns a color based on the match type.
4.  Highlighting is updated whenever a row changes (dirty check).

### Search
Implements a linear search:
*   Scans forward/backward from the cursor position.
*   Wraps around to the beginning/end of the document.
*   Highlights the match and jumps the cursor to it.

---

## 5. User Manual

### Installation
1.  **Dependencies**: Rust & Cargo.
2.  **Install**:
    ```bash
    ./install.sh
    ```
    This compiles the binary and sets up your global configuration.

### Basic Workflow
1.  **Open File**: `meow filename.rs`
2.  **Edit**: Press `i` to type. Press `Esc` to stop.
3.  **Save**: Press `:` then `w`, then `Enter`.
4.  **Quit**: Press `:` then `q`, then `Enter`.

### Key Bindings
*   **Movement**: `h` (Left), `j` (Down), `k` (Up), `l` (Right)
*   **Editing**: `x` (Delete char), `d` (Delete selection), `y` (Copy), `p` (Paste).

---

## 6. Future Roadmap
Potential areas for study and improvement:

*   **LSP Support**: Integrate with Language Servers for real auto-completion and error checking.
*   **Tree-sitter**: Used for more robust parsing-based syntax highlighting.

---

*This guide serves as a comprehensive manual for understanding, using, and developing the Meow Text Editor.*

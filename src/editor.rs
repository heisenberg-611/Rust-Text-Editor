use crate::config::Config;
use crate::document::{Document, SearchDirection};
use crate::terminal::Terminal;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::io;
use std::time::{Duration, Instant};

fn parse_hex_color(hex: &str) -> Color {
    if hex.len() != 7 || !hex.starts_with('#') {
        return Color::Reset;
    }

    let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);

    Color::Rgb(r, g, b)
}

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Visual,
    Search,
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: String,
    status_time: Instant,
    mode: Mode,
    selection_start: Option<Position>,
    mouse_drag_start: Option<Position>,
    last_search_query: Option<String>,
    clipboard: Option<arboard::Clipboard>,
    #[allow(dead_code)]
    config: Config,
    command_buffer: String,
}

impl Editor {
    pub fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let mut document = Document::default();
        if let Some(filename) = args.get(1) {
            if let Ok(doc) = Document::open(filename) {
                document = doc;
            } else {
                // If file doesn't exist, we'll create it on save
                document.file_name = Some(filename.clone());
            }
        }

        let config = Config::load();
        let clipboard = arboard::Clipboard::new().ok();

        Self {
            should_quit: false,
            terminal: Terminal::new().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
            status_message: String::new(),
            status_time: Instant::now(),
            mode: Mode::Normal,
            selection_start: None,
            mouse_drag_start: None,
            last_search_query: None,
            clipboard,
            config,
            command_buffer: String::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Err(e) = self.refresh_screen() {
                die(e);
            }
            if self.should_quit {
                break;
            }
            if let Err(e) = self.process_keypress() {
                die(e);
            }
        }
        self.terminal.stop().unwrap();
    }

    fn refresh_screen(&mut self) -> Result<(), io::Error> {
        let mode = self.mode;
        let command_buf = self.command_buffer.clone();
        let status_msg = self.status_message.clone();
        let doc_len = self.document.len();
        let filename = self
            .document
            .file_name
            .clone()
            .unwrap_or("[No Name]".to_string());

        // Calculate viewport
        let terminal_size = self.terminal.backend.size()?;
        let height = terminal_size.height as usize;
        let width = terminal_size.width as usize;

        let offset_x = self.offset.x;
        let offset_y = self.offset.y;

        let cursor_x = self.cursor_position.x.saturating_sub(offset_x);
        let cursor_y = self.cursor_position.y.saturating_sub(offset_y);

        // Calculate gutter width for line numbers
        let gutter_width = if self.config.editor.line_numbers {
            // Width of line number + 1 space padding
            let digits = doc_len.to_string().len();
            digits + 2 // " 1 " style padding
        } else {
            0
        };

        let text_width = width.saturating_sub(gutter_width);

        // Prepare text to render
        let mut lines = Vec::new();
        for y in 0..height.saturating_sub(2) {
            // Reserve space for status bar
            let file_row = y + offset_y;
            if file_row < doc_len {
                if let Some(row) = self.document.row(file_row) {
                    let mut spans = Vec::new();

                    if self.config.editor.line_numbers {
                        let line_num = file_row + 1;
                        let digits = doc_len.to_string().len();
                        // Right align line number
                        let default_style = Style::default()
                            .fg(Color::DarkGray)
                            .bg(parse_hex_color(&self.config.theme.background));
                        // Use background color for gutter to match rendering or distinct?
                        // Usually gutter has same bg or slightly different. Let's use theme background for now.
                        let gutter_str = format!("{:>width$} ", line_num, width = digits + 1);
                        spans.push(Span::styled(gutter_str, default_style));
                    }

                    let row_content = row.render(offset_x, offset_x + text_width);

                    if self.mode == Mode::Visual {
                        if let Some(start_pos) = self.selection_start {
                            let (start, end) = if start_pos.y < self.cursor_position.y
                                || (start_pos.y == self.cursor_position.y
                                    && start_pos.x <= self.cursor_position.x)
                            {
                                (start_pos, self.cursor_position)
                            } else {
                                (self.cursor_position, start_pos)
                            };

                            let current_row_idx = file_row;
                            if current_row_idx < start.y || current_row_idx > end.y {
                                spans.push(Span::raw(row_content));
                            } else {
                                // This row is part of the selection range
                                let row_len = row.len();
                                let _sel_start_x = if current_row_idx == start.y {
                                    start.x
                                } else {
                                    0
                                };
                                let _sel_end_x = if current_row_idx == end.y {
                                    end.x.min(row_len)
                                } else {
                                    row_len
                                };

                                // Adjust for viewport offset
                                // let _render_start_x = sel_start_x.saturating_sub(offset_x);
                                // let _render_end_x = sel_end_x.saturating_sub(offset_x);

                                // We need to split row_content string into chars to handle multibyte correctly and indices
                                // Ideally we would work with byte indices or char indices from row.render
                                // For simplicity, let's just highlight the whole line if fully selected,
                                // or try to substring. Note: row.render returns a substring of the content.

                                // Let's iterate chars of render result
                                let mut current_x = offset_x;
                                let mut normal_before = String::new();
                                let mut selected = String::new();
                                let mut normal_after = String::new();

                                for c in row.content.chars() {
                                    if current_x >= offset_x + width {
                                        break;
                                    }
                                    if current_x >= offset_x {
                                        // Visible char
                                        let is_selected = if current_row_idx > start.y
                                            && current_row_idx < end.y
                                        {
                                            true
                                        } else if current_row_idx == start.y
                                            && current_row_idx == end.y
                                        {
                                            current_x >= start.x && current_x <= end.x
                                        // Inclusive end for cursor feel? Standard VIM is usually exclusive on end or inclusive depending on settings. Let's do inclusive of cursor.
                                        } else if current_row_idx == start.y {
                                            current_x >= start.x
                                        } else if current_row_idx == end.y {
                                            current_x <= end.x
                                        } else {
                                            false
                                        };

                                        if is_selected {
                                            selected.push(c);
                                        } else if !selected.is_empty() && normal_after.is_empty() {
                                            normal_after.push(c);
                                        } else if selected.is_empty() {
                                            normal_before.push(c);
                                        } else {
                                            normal_after.push(c);
                                        }
                                    }
                                    current_x += 1;
                                }

                                if !normal_before.is_empty() {
                                    spans.push(Span::styled(
                                        normal_before,
                                        Style::default()
                                            .fg(parse_hex_color(&self.config.theme.foreground)),
                                    ));
                                }
                                if !selected.is_empty() {
                                    spans.push(Span::styled(
                                        selected,
                                        Style::default()
                                            .bg(parse_hex_color(&self.config.theme.selection_bg))
                                            .fg(parse_hex_color(&self.config.theme.foreground)),
                                    ));
                                }
                                if !normal_after.is_empty() {
                                    spans.push(Span::styled(
                                        normal_after,
                                        Style::default()
                                            .fg(parse_hex_color(&self.config.theme.foreground)),
                                    ));
                                }

                                // Fallback if logic failed (e.g empty selection that implies cursor pos)
                                if spans.is_empty() {
                                    spans.push(Span::styled(
                                        row_content,
                                        Style::default()
                                            .fg(parse_hex_color(&self.config.theme.foreground)),
                                    ));
                                }
                            }
                        } else {
                            spans.push(Span::styled(
                                row_content,
                                Style::default().fg(parse_hex_color(&self.config.theme.foreground)),
                            ));
                        }
                    } else {
                        spans.push(Span::styled(
                            row_content,
                            Style::default().fg(parse_hex_color(&self.config.theme.foreground)),
                        ));
                    }
                    lines.push(Line::from(spans));
                }
            } else if file_row == doc_len && doc_len == 0 {
                // Empty buffer
                let mut empty_line_spans = Vec::new();
                if self.config.editor.line_numbers {
                    let digits = 1; // 0 len -> 1 digit
                    let gutter_str = format!("{:>width$} ", 1, width = digits + 1);
                    empty_line_spans.push(Span::styled(
                        gutter_str,
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                empty_line_spans.push(Span::styled(
                    "~ empty buffer ~",
                    Style::default().fg(Color::DarkGray),
                ));
                lines.push(Line::from(empty_line_spans));
            } else {
                lines.push(Line::styled("~", Style::default().fg(Color::DarkGray)));
            }
        }

        self.terminal.backend.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(f.area());

            let text_area = Paragraph::new(lines)
                .style(Style::default().bg(parse_hex_color(&self.config.theme.background)));
            f.render_widget(text_area, chunks[0]);

            // Status Bar
            let mode_str = match mode {
                Mode::Normal => "NORMAL",
                Mode::Insert => "INSERT",
                Mode::Command => "COMMAND",
                Mode::Visual => "VISUAL",
                Mode::Search => "SEARCH",
            };
            let status_text = format!(
                " {} | {} | Lines: {} | Bytes: {}",
                mode_str,
                filename,
                doc_len,
                self.document.size_bytes()
            );
            let status_bar = Paragraph::new(status_text).style(
                Style::default()
                    .bg(parse_hex_color(&self.config.theme.selection_bg))
                    .fg(parse_hex_color(&self.config.theme.foreground)),
            );
            f.render_widget(status_bar, chunks[1]);

            // Command/Message Line
            let cmd_text = match mode {
                Mode::Command => format!(":{}", command_buf),
                Mode::Search => format!("/{}", command_buf),
                _ => {
                    if status_msg.is_empty()
                        || Instant::now().duration_since(self.status_time) > Duration::from_secs(5)
                    {
                        String::new()
                    } else {
                        status_msg
                    }
                }
            };

            f.render_widget(Paragraph::new(cmd_text), chunks[2]);

            if mode != Mode::Command && mode != Mode::Search {
                f.set_cursor_position((
                    chunks[0].x + gutter_width as u16 + cursor_x as u16,
                    chunks[0].y + cursor_y as u16,
                ));
            } else {
                f.set_cursor_position((chunks[2].x + 1 + command_buf.len() as u16, chunks[2].y));
            }
        })?;
        Ok(())
    }

    fn process_keypress(&mut self) -> Result<(), io::Error> {
        if crossterm::event::poll(Duration::from_millis(100))? {
            let event = crossterm::event::read()?;
            match event {
                crossterm::event::Event::Key(key) => match self.mode {
                    Mode::Normal => self.process_normal_mode(key),
                    Mode::Insert => self.process_insert_mode(key),
                    Mode::Command => self.process_command_mode(key),
                    Mode::Visual => self.process_visual_mode(key),
                    Mode::Search => self.process_search_mode(key),
                },
                crossterm::event::Event::Mouse(mouse_event) => {
                    self.process_mouse(mouse_event);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn process_mouse(&mut self, event: MouseEvent) {
        let x = event.column as usize;
        let y = event.row as usize;
        let terminal_height = self.terminal.backend.size().unwrap().height as usize;

        let doc_len = self.document.len();
        let gutter_width = if self.config.editor.line_numbers {
            doc_len.to_string().len() + 2
        } else {
            0
        };

        // Check if click is within text area (simplified)
        if y < terminal_height.saturating_sub(2) {
            let doc_x = if x >= gutter_width {
                self.offset.x + (x - gutter_width)
            } else {
                self.offset.x // Click on gutter -> start of line?
            };
            let doc_y = self.offset.y + y;

            // Ignore click on gutter? or select line?
            // For now let's just allow it to move cursor to start if clicked on gutter (x < gutter_width -> doc_x = offset_x)

            match event.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    self.move_cursor_absolute(doc_x, doc_y);
                    self.mode = Mode::Normal;
                    self.selection_start = None;
                    self.mouse_drag_start = Some(self.cursor_position);
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    self.move_cursor_absolute(doc_x, doc_y);
                    if self.mouse_drag_start.is_some() {
                        if self.mode == Mode::Normal {
                            self.mode = Mode::Visual;
                            self.selection_start = self.mouse_drag_start;
                        }
                    }
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    self.mouse_drag_start = None;
                }
                MouseEventKind::ScrollUp => {
                    self.move_cursor(0, -3);
                }
                MouseEventKind::ScrollDown => {
                    self.move_cursor(0, 3);
                }
                _ => {}
            }
        }
    }

    fn process_visual_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.selection_start = None;
            }
            KeyCode::Char('h') => self.move_cursor(-1, 0),
            KeyCode::Char('j') => self.move_cursor(0, 1),
            KeyCode::Char('k') => self.move_cursor(0, -1),
            KeyCode::Char('l') => self.move_cursor(1, 0),
            KeyCode::Char('y') => {
                if let Some(start) = self.selection_start {
                    let content = self.document.get_substring(&start, &self.cursor_position);
                    if let Some(cb) = &mut self.clipboard {
                        let _ = cb.set_text(content);
                    }
                }
                self.mode = Mode::Normal;
                self.selection_start = None;
                self.set_status_message("Yanked!".to_string());
            }
            KeyCode::Char('d') => {
                if let Some(start) = self.selection_start {
                    self.document.delete_range(&start, &self.cursor_position);
                    // Move cursor to start of deletion
                    let (new_pos, _) = if start.y < self.cursor_position.y
                        || (start.y == self.cursor_position.y && start.x <= self.cursor_position.x)
                    {
                        (start, self.cursor_position)
                    } else {
                        (self.cursor_position, start)
                    };
                    self.move_cursor_absolute(new_pos.x, new_pos.y);
                }
                self.mode = Mode::Normal;
                self.selection_start = None;
                self.set_status_message("Deleted".to_string());
            }
            KeyCode::Char('x') => {
                if let Some(start) = self.selection_start {
                    let content = self.document.get_substring(&start, &self.cursor_position);
                    if let Some(cb) = &mut self.clipboard {
                        let _ = cb.set_text(content);
                    }
                    self.document.delete_range(&start, &self.cursor_position);
                    // Move cursor to start of deletion
                    let (new_pos, _) = if start.y < self.cursor_position.y
                        || (start.y == self.cursor_position.y && start.x <= self.cursor_position.x)
                    {
                        (start, self.cursor_position)
                    } else {
                        (self.cursor_position, start)
                    };
                    self.move_cursor_absolute(new_pos.x, new_pos.y);
                }
                self.mode = Mode::Normal;
                self.selection_start = None;
                self.set_status_message("Cut!".to_string());
            }
            KeyCode::Left => self.move_cursor(-1, 0),
            KeyCode::Right => self.move_cursor(1, 0),
            KeyCode::Up => self.move_cursor(0, -1),
            KeyCode::Down => self.move_cursor(0, 1),
            _ => {}
        }
    }

    fn process_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true, // Quick quit for debugging
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            }
            KeyCode::Char('h') => self.move_cursor(-1, 0),
            KeyCode::Char('j') => self.move_cursor(0, 1),
            KeyCode::Char('k') => self.move_cursor(0, -1),
            KeyCode::Char('l') => self.move_cursor(1, 0),
            KeyCode::Char('x') => self.document.delete(&self.cursor_position),
            KeyCode::Char('v') => {
                self.mode = Mode::Visual;
                self.selection_start = Some(self.cursor_position);
            }
            KeyCode::Char('p') => {
                if let Some(cb) = &mut self.clipboard {
                    if let Ok(content) = cb.get_text() {
                        for c in content.chars() {
                            self.document.insert(&self.cursor_position, c);
                            if c == '\n' {
                                let pos = Position {
                                    x: 0,
                                    y: self.cursor_position.y + 1,
                                };
                                self.move_cursor_absolute(pos.x, pos.y);
                            } else {
                                self.move_cursor(1, 0);
                            }
                        }
                    }
                }
            }
            KeyCode::Char('/') => {
                self.mode = Mode::Search;
                self.command_buffer.clear();
            }
            KeyCode::Char('n') => {
                if let Some(query) = self.last_search_query.clone() {
                    self.run_search(&query, SearchDirection::Forward);
                }
            }
            KeyCode::Char('N') => {
                if let Some(query) = self.last_search_query.clone() {
                    self.run_search(&query, SearchDirection::Backward);
                }
            }
            KeyCode::Left => self.move_cursor(-1, 0),
            KeyCode::Right => self.move_cursor(1, 0),
            KeyCode::Up => self.move_cursor(0, -1),
            KeyCode::Down => self.move_cursor(0, 1),
            _ => {}
        }
    }

    fn process_insert_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(1, 0);
            }
            KeyCode::Enter => {
                self.document.insert_newline(&self.cursor_position);
                self.move_cursor_absolute(0, self.cursor_position.y + 1);
            }
            KeyCode::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    if self.cursor_position.x > 0 {
                        self.move_cursor(-1, 0);
                        self.document.delete(&self.cursor_position);
                    } else if self.cursor_position.y > 0 {
                        let prev_y = self.cursor_position.y - 1;
                        if let Some(row) = self.document.row(prev_y) {
                            let len = row.len();
                            self.move_cursor_absolute(len, prev_y);
                            self.document.delete(&self.cursor_position);
                        }
                    }
                }
            }
            KeyCode::Left => self.move_cursor(-1, 0),
            KeyCode::Right => self.move_cursor(1, 0),
            KeyCode::Up => self.move_cursor(0, -1),
            KeyCode::Down => self.move_cursor(0, 1),
            _ => {}
        }
    }

    fn process_command_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Enter => {
                self.execute_command();
                self.mode = Mode::Normal;
            }
            _ => {}
        }
    }

    fn process_search_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
            }
            KeyCode::Enter => {
                let query = self.command_buffer.clone();
                self.last_search_query = Some(query.clone());
                self.mode = Mode::Normal;
                self.run_search(&query, SearchDirection::Forward);
                self.command_buffer.clear();
            }
            _ => {}
        }
    }

    fn run_search(&mut self, query: &str, direction: SearchDirection) {
        let start_pos = match direction {
            SearchDirection::Forward => {
                // Forward search should start AFTER current char to find next match
                // But wait, our find logic scans matches >= start_x.
                // So if we are on match, we find it again.
                // So for 'n', we should advance by 1?
                // Or we pass cursor pos, and `find` is inclusive.
                // We want strictly next.

                // Let's modify start pos.
                if self.cursor_position.x < usize::MAX {
                    // Safety match
                    Position {
                        x: self.cursor_position.x + 1,
                        y: self.cursor_position.y,
                    }
                } else {
                    self.cursor_position
                }
            }
            SearchDirection::Backward => {
                // For backward, we want match < current_x.
                // Our find logic looks for matches < limit_x
                // So passing cursor_position is correct (exclusive logic there)
                self.cursor_position
            }
        };

        if let Some(pos) = self.document.find(query, &start_pos, direction) {
            self.move_cursor_absolute(pos.x, pos.y);
            self.selection_start = None; // clear selection if any
            self.set_status_message(String::new());
        } else {
            self.set_status_message(format!("Pattern not found: {}", query));
        }
    }

    fn execute_command(&mut self) {
        let cmd = self.command_buffer.trim();
        if cmd == "q" {
            self.should_quit = true;
        } else if cmd == "w" {
            if let Err(e) = self.document.save() {
                self.set_status_message(format!("Error: {}", e));
            } else {
                self.set_status_message(format!("Written {} bytes", self.document.size_bytes()));
            }
        } else if cmd == "wq" {
            let _ = self.document.save();
            self.should_quit = true;
        } else {
            self.set_status_message(format!("Not an editor command: {}", cmd));
        }
    }

    fn set_status_message(&mut self, msg: String) {
        self.status_message = msg;
        self.status_time = Instant::now();
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) {
        let x = self.cursor_position.x as i32 + dx;
        let y = self.cursor_position.y as i32 + dy;
        self.move_cursor_absolute(x.max(0) as usize, y.max(0) as usize);
    }

    fn move_cursor_absolute(&mut self, x: usize, y: usize) {
        let height = self.document.len();
        let mut y = y;
        if y >= height {
            y = height;
        }
        self.cursor_position.y = y;

        let row_len = if y < height {
            self.document.row(y).unwrap().len()
        } else {
            0
        };

        let mut x = x;
        if x > row_len {
            x = row_len;
        }
        self.cursor_position.x = x;
        self.scroll();
    }

    fn scroll(&mut self) {
        let size = self.terminal.backend.size().unwrap();
        let height = size.height as usize;
        let mut width = size.width as usize;

        if self.config.editor.line_numbers {
            let digits = self.document.len().to_string().len();
            width = width.saturating_sub(digits + 2);
        }

        if self.cursor_position.y < self.offset.y {
            self.offset.y = self.cursor_position.y;
        } else if self.cursor_position.y >= self.offset.y.saturating_add(height).saturating_sub(2) {
            self.offset.y = self
                .cursor_position
                .y
                .saturating_sub(height)
                .saturating_add(2)
                .saturating_add(1);
        }

        if self.cursor_position.x < self.offset.x {
            self.offset.x = self.cursor_position.x;
        } else if self.cursor_position.x >= self.offset.x.saturating_add(width) {
            self.offset.x = self
                .cursor_position
                .x
                .saturating_sub(width)
                .saturating_add(1);
        }
    }
}

fn die(e: io::Error) {
    panic!("{}", e);
}

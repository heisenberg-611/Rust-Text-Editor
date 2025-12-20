use crate::config::Config;
use crate::document::Document;
use crate::terminal::Terminal;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind, MouseButton};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use std::io;
use std::time::{Duration, Instant};

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
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: String,
    #[allow(dead_code)]
    status_time: Instant,
    mode: Mode,
    selection_start: Option<Position>,
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
        let filename = self.document.file_name.clone().unwrap_or("[No Name]".to_string());
        
        // Calculate viewport
        let terminal_size = self.terminal.backend.size()?;
        let height = terminal_size.height as usize;
        let width = terminal_size.width as usize;
        
        let offset_x = self.offset.x;
        let offset_y = self.offset.y;
        
        let cursor_x = self.cursor_position.x.saturating_sub(offset_x);
        let cursor_y = self.cursor_position.y.saturating_sub(offset_y);

        // Prepare text to render
        let mut lines = Vec::new();
        for y in 0..height.saturating_sub(2) { // Reserve space for status bar
             let file_row = y + offset_y;
             if file_row < doc_len {
                 if let Some(row) = self.document.row(file_row) {
                     let mut spans = Vec::new();
                     let row_content = row.render(offset_x, offset_x + width);
                     
                     if self.mode == Mode::Visual {
                         if let Some(start_pos) = self.selection_start {
                             let (start, end) = if start_pos.y < self.cursor_position.y || (start_pos.y == self.cursor_position.y && start_pos.x <= self.cursor_position.x) {
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
                                 let sel_start_x = if current_row_idx == start.y { start.x } else { 0 };
                                 let sel_end_x = if current_row_idx == end.y { end.x.min(row_len) } else { row_len };
                                 
                                 // Adjust for viewport offset
                                 let render_start_x = sel_start_x.saturating_sub(offset_x);
                                 let render_end_x = sel_end_x.saturating_sub(offset_x);
                                 
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
                                     if current_x >= offset_x + width { break; }
                                     if current_x >= offset_x {
                                         // Visible char
                                         let is_selected = if current_row_idx > start.y && current_row_idx < end.y {
                                             true
                                         } else if current_row_idx == start.y && current_row_idx == end.y {
                                             current_x >= start.x && current_x <= end.x // Inclusive end for cursor feel? Standard VIM is usually exclusive on end or inclusive depending on settings. Let's do inclusive of cursor.
                                         } else if current_row_idx == start.y {
                                             current_x >= start.x
                                         } else if current_row_idx == end.y {
                                             current_x <= end.x
                                         } else {
                                             false
                                         };

                                         if is_selected {
                                             selected.push(c);
                                         } else if !selected.is_empty() && normal_after.is_empty(){
                                              normal_after.push(c);
                                         } else if selected.is_empty() {
                                              normal_before.push(c);
                                         } else {
                                              normal_after.push(c);
                                         }
                                     }
                                     current_x += 1;
                                 }
                                 
                                 if !normal_before.is_empty() { spans.push(Span::raw(normal_before)); }
                                 if !selected.is_empty() { spans.push(Span::styled(selected, Style::default().bg(Color::Blue))); }
                                 if !normal_after.is_empty() { spans.push(Span::raw(normal_after)); }
                                 
                                 // Fallback if logic failed (e.g empty selection that implies cursor pos)
                                 if spans.is_empty() {
                                     spans.push(Span::raw(row_content));
                                 }
                             }
                         } else {
                              spans.push(Span::raw(row_content));
                         }
                     } else {
                         spans.push(Span::raw(row_content));
                     }
                     lines.push(Line::from(spans));
                 }
             } else if file_row == doc_len && doc_len == 0 {
                  lines.push(Line::from("~ empty buffer ~"));
             } else {
                 lines.push(Line::from("~"));
             }
        }

        self.terminal.backend.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ].as_ref())
                .split(f.area());

            let text_area = Paragraph::new(lines);
            f.render_widget(text_area, chunks[0]);

            // Status Bar
            let mode_str = match mode {
                Mode::Normal => "NORMAL",
                Mode::Insert => "INSERT",
                Mode::Command => "COMMAND",
                Mode::Visual => "VISUAL",
            };
            let status_text = format!(" {} | {} | Lines: {}", mode_str, filename, doc_len);
            let status_bar = Paragraph::new(status_text)
                .style(Style::default().bg(Color::Blue).fg(Color::White));
            f.render_widget(status_bar, chunks[1]);
            
            // Command/Message Line
            let cmd_text = if mode == Mode::Command {
                format!(":{}", command_buf)
            } else {
                status_msg
            };
            f.render_widget(Paragraph::new(cmd_text), chunks[2]);
            
            if mode != Mode::Command {
                f.set_cursor_position(
                    (chunks[0].x + cursor_x as u16,
                    chunks[0].y + cursor_y as u16),
                );
            } else {
                 f.set_cursor_position(
                    (chunks[2].x + 1 + command_buf.len() as u16,
                     chunks[2].y)
                );
            }
        })?;
        Ok(())
    }

    fn process_keypress(&mut self) -> Result<(), io::Error> {
        if crossterm::event::poll(Duration::from_millis(100))? {
            let event = crossterm::event::read()?;
            match event {
                crossterm::event::Event::Key(key) => {
                    match self.mode {
                        Mode::Normal => self.process_normal_mode(key),
                        Mode::Insert => self.process_insert_mode(key),
                        Mode::Command => self.process_command_mode(key),
                        Mode::Visual => self.process_visual_mode(key),
                    }
                }
                crossterm::event::Event::Mouse(mouse_event) => {
                     self.process_mouse(mouse_event);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn process_mouse(&mut self, event: MouseEvent) {
        if let MouseEventKind::Down(MouseButton::Left) = event.kind {
             let x = event.column as usize;
             let y = event.row as usize;
             // We need to adjust for offsets and UI layout (e.g. status bar is at bottom)
             // The text area is at chunks[0]
             
             // Simplification: We blindly assume text area starts at 0,0 and takes up most space.
             // But we have chunks[0] height. We know status bar is last 2 lines.
             // We should check if y is within the text area.
             
             let terminal_height = self.terminal.backend.size().unwrap().height as usize;
             if y < terminal_height.saturating_sub(2) {
                  let doc_x = self.offset.x + x;
                  let doc_y = self.offset.y + y;
                  self.move_cursor_absolute(doc_x, doc_y);
                  
                  if self.mode == Mode::Visual {
                      // If clicking in visual mode, maybe update selection end?
                      // For now let's just move cursor. Standard behavior: click resets selection unless Shift held.
                      // Let's reset mode to Normal if mouse clicked? Or keep visual and update end?
                      // Let's go to Normal mode on click for simplicity.
                      self.mode = Mode::Normal;
                      self.selection_start = None;
                  }
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
                self.status_message = "Yanked!".to_string();
            }
            KeyCode::Char('d') | KeyCode::Char('x') => {
                 if let Some(start) = self.selection_start {
                    let content = self.document.get_substring(&start, &self.cursor_position);
                    if let Some(cb) = &mut self.clipboard {
                        let _ = cb.set_text(content);
                    }
                    self.document.delete_range(&start, &self.cursor_position);
                    // Move cursor to start of deletion
                    let (new_pos, _) = if start.y < self.cursor_position.y || (start.y == self.cursor_position.y && start.x <= self.cursor_position.x) {
                        (start, self.cursor_position)
                    } else {
                        (self.cursor_position, start)
                    };
                    self.move_cursor_absolute(new_pos.x, new_pos.y);
                }
                self.mode = Mode::Normal;
                self.selection_start = None;
                self.status_message = "Cut!".to_string();
            }
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
                                 let pos = Position { x: 0, y: self.cursor_position.y + 1 };
                                 self.move_cursor_absolute(pos.x, pos.y);
                             } else {
                                 self.move_cursor(1, 0);
                             }
                         }
                    }
                }
            }
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
                     self.move_cursor(-1, 0);
                     self.document.delete(&self.cursor_position);
                 }
            }
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

    fn execute_command(&mut self) {
        let cmd = self.command_buffer.trim();
        if cmd == "q" {
            self.should_quit = true;
        } else if cmd == "w" {
            if let Err(e) = self.document.save() {
                self.status_message = format!("Error: {}", e);
            } else {
                self.status_message = "Written".to_string();
            }
        } else if cmd == "wq" {
             let _ = self.document.save();
             self.should_quit = true;
        } else {
            self.status_message = format!("Not an editor command: {}", cmd);
        }
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
        let width = size.width as usize;
        
        if self.cursor_position.y < self.offset.y {
            self.offset.y = self.cursor_position.y;
        } else if self.cursor_position.y >= self.offset.y.saturating_add(height).saturating_sub(2) {
             self.offset.y = self.cursor_position.y.saturating_sub(height).saturating_add(2).saturating_add(1);
        }
        
        if self.cursor_position.x < self.offset.x {
             self.offset.x = self.cursor_position.x;
        } else if self.cursor_position.x >= self.offset.x.saturating_add(width) {
             self.offset.x = self.cursor_position.x.saturating_sub(width).saturating_add(1);
        }
    }

}


fn die(e: io::Error) {
    panic!("{}", e);
}

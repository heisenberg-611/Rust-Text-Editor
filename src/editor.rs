use crate::config::Config;
use crate::document::Document;
use crate::terminal::Terminal;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::io;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
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

        Self {
            should_quit: false,
            terminal: Terminal::new().expect("Failed to initialize terminal"),
            cursor_position: Position::default(),
            offset: Position::default(),
            document,
            status_message: String::new(),
            status_time: Instant::now(),
            mode: Mode::Normal,
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
                     lines.push(Line::from(row.render(offset_x, offset_x + width)));
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
                .split(f.size());

            let text_area = Paragraph::new(lines);
            f.render_widget(text_area, chunks[0]);

            // Status Bar
            let mode_str = match mode {
                Mode::Normal => "NORMAL",
                Mode::Insert => "INSERT",
                Mode::Command => "COMMAND",
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
            
            // Cursor
            if mode != Mode::Command {
                f.set_cursor(
                    (chunks[0].x + cursor_x as u16),
                    (chunks[0].y + cursor_y as u16),
                );
            } else {
                 f.set_cursor(
                    (chunks[2].x + 1 + command_buf.len() as u16),
                     chunks[2].y
                );
            }
        })?;
        Ok(())
    }

    fn process_keypress(&mut self) -> Result<(), io::Error> {
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match self.mode {
                    Mode::Normal => self.process_normal_mode(key),
                    Mode::Insert => self.process_insert_mode(key),
                    Mode::Command => self.process_command_mode(key),
                }
            }
        }
        Ok(())
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

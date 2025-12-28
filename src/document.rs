use crate::editor::Position;
use crate::row::Row;
use ropey::Rope;
use std::fs;
use std::io::{BufWriter, Error};

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

pub struct Document {
    pub content: Rope,
    pub file_name: Option<String>,
    pub dirty: bool,
    pub syntax: &'static crate::syntax::Syntax,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            content: Rope::new(),
            file_name: None,
            dirty: false,
            syntax: crate::syntax::Syntax::default_ref(),
        }
    }
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let content = Rope::from_str(&contents);
        let syntax = crate::syntax::Syntax::select(filename);

        Ok(Self {
            content,
            file_name: Some(filename.to_string()),
            dirty: false,
            syntax,
        })
    }

    pub fn save(&self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let file = fs::File::create(file_name)?;
            let mut writer = BufWriter::new(file);
            self.content.write_to(&mut writer)?;
        }
        Ok(())
    }

    pub fn row(&self, index: usize) -> Option<Row> {
        if index >= self.len() {
            return None;
        }

        let line = self.content.line(index);
        // Ropey lines include the newline character, but Row expects without.
        // We need to strip it.
        let line_cow = line.to_string(); // Convert to String (owned)
        // Check if ends with newline and remove it
        let content = if line_cow.ends_with('\n') {
            let mut s = line_cow;
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
            s
        } else {
            line_cow
        };

        let mut row = Row::from(content.as_str());
        row.update_highlighting(self.syntax);
        Some(row)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.content.len_chars() == 0
    }

    pub fn len(&self) -> usize {
        self.content.len_lines()
    }

    pub fn size_bytes(&self) -> usize {
        self.content.len_bytes()
    }

    pub fn insert(&mut self, at: &crate::editor::Position, c: char) {
        let char_idx = self.position_to_char_idx(at);
        if char_idx <= self.content.len_chars() {
            self.content.insert_char(char_idx, c);
            self.dirty = true;
        }
    }

    pub fn insert_newline(&mut self, at: &crate::editor::Position) {
        self.insert(at, '\n');
    }

    pub fn delete(&mut self, at: &crate::editor::Position) {
        let char_idx = self.position_to_char_idx(at);
        if char_idx < self.content.len_chars() {
            self.content.remove(char_idx..char_idx + 1);
            self.dirty = true;
        }
    }

    // Helper to convert Position (x, y) to absolute char index for Rope
    fn position_to_char_idx(&self, pos: &Position) -> usize {
        if pos.y >= self.len() {
            return self.content.len_chars();
        }
        let line_char_idx = self.content.line_to_char(pos.y);

        // Ensure we don't go beyond the line (including newline)
        // Note: x is visual index, but we assume 1 char = 1 index for now (UTF-8).
        // Rope handles UTF-8 correctly.

        // If x is past the end of the line (excluding newline), we clamp it?
        // But insert needs to be able to append.

        // If line ends with \n, valid x indices are 0..len-1 (for insertion before \n) ?
        // Usually editors allow cursor to be at the newline char (which effectively appends to line).

        // actually, Ropey::line includes \n.
        // line_char_idx + pos.x.

        // We verify that pos.x is valid for this line.
        // Row len doesn't include \n.
        // So pos.x <= row_len.

        if pos.x + line_char_idx > self.content.len_chars() {
            return self.content.len_chars();
        }

        line_char_idx + pos.x
    }

    pub fn get_substring(&self, start: &Position, end: &Position) -> String {
        let start_idx = self.position_to_char_idx(start);
        let end_idx = self.position_to_char_idx(end);
        let (start_idx, end_idx) = if start_idx <= end_idx {
            (start_idx, end_idx)
        } else {
            (end_idx, start_idx)
        };

        // Inclusive? get_substring usually implies range.
        // Original implementation was inclusive of end column?
        // Original: `take(end_x - start_x + 1)`. Yes inclusive.
        // So we need end_idx + 1, but be careful of bounds.

        let len = self.content.len_chars();
        let end_idx = (end_idx + 1).min(len);

        if start_idx >= len {
            return String::new();
        }

        self.content.slice(start_idx..end_idx).to_string()
    }

    pub fn delete_range(&mut self, start: &Position, end: &Position) {
        let start_idx = self.position_to_char_idx(start);
        let end_idx = self.position_to_char_idx(end);
        let (start_idx, end_idx) = if start_idx <= end_idx {
            (start_idx, end_idx)
        } else {
            (end_idx, start_idx)
        };

        // Inclusive delete logic matches get_substring?
        // Original `delete` deleted char at cursor.
        // `delete_range` deleted from start to end inclusive.

        let len = self.content.len_chars();
        let end_idx = (end_idx + 1).min(len);

        if start_idx < len {
            self.content.remove(start_idx..end_idx);
            self.dirty = true;
        }
    }

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if query.is_empty() {
            return None;
        }

        let start_char_idx = self.position_to_char_idx(at);
        let content_str = self.content.to_string(); // Converting whole rope to string is expensive but simplest for search now.
        // Ropey doesn't have built-in search yet? It does have iterators.
        // For efficiency we should iterate chunks, but for now `to_string` is acceptable for MVP migration.

        match direction {
            SearchDirection::Forward => {
                // Search from start_char_idx
                if let Some(idx) = content_str[start_char_idx..].find(query) {
                    let found_idx = start_char_idx + idx;
                    // Convert back to Position
                    let line_idx = self.content.char_to_line(found_idx);
                    let line_start = self.content.line_to_char(line_idx);
                    let x = found_idx - line_start;
                    return Some(Position { x, y: line_idx });
                } else {
                    // Wrap around? Original implementation wrapped.
                    if let Some(idx) = content_str.find(query) {
                        let found_idx = idx;
                        let line_idx = self.content.char_to_line(found_idx);
                        let line_start = self.content.line_to_char(line_idx);
                        let x = found_idx - line_start;
                        return Some(Position { x, y: line_idx });
                    }
                }
            }
            SearchDirection::Backward => {
                // Search before start_char_idx
                // find (forward) then filter? or rfind?
                // `rfind` searches from right.
                if let Some(idx) = content_str[..start_char_idx].rfind(query) {
                    let found_idx = idx;
                    let line_idx = self.content.char_to_line(found_idx);
                    let line_start = self.content.line_to_char(line_idx);
                    let x = found_idx - line_start;
                    return Some(Position { x, y: line_idx });
                } else {
                    // Wrap around to end
                    if let Some(idx) = content_str.rfind(query) {
                        let found_idx = idx;
                        let line_idx = self.content.char_to_line(found_idx);
                        let line_start = self.content.line_to_char(line_idx);
                        let x = found_idx - line_start;
                        return Some(Position { x, y: line_idx });
                    }
                }
            }
        }

        None
    }
}

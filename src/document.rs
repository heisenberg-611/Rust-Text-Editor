use crate::editor::Position;
use crate::row::Row;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    pub rows: Vec<Row>,
    pub file_name: Option<String>,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        Ok(Self {
            rows,
            file_name: Some(filename.to_string()),
        })
    }

    pub fn save(&self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.content.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
        Ok(())
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn insert(&mut self, at: &crate::editor::Position, c: char) {
        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else if at.y < self.len() {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }
    }

    pub fn insert_newline(&mut self, at: &crate::editor::Position) {
        if at.y > self.len() {
            return;
        }
        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        }
        let new_row = self.rows.get_mut(at.y).unwrap().split(at.x);
        self.rows.insert(at.y + 1, new_row);
    }

    pub fn delete(&mut self, at: &crate::editor::Position) {
        let len = self.len();
        if at.y >= len {
            return;
        }
        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }
    }

    pub fn get_substring(&self, start: &Position, end: &Position) -> String {
        let (start, end) = if start.y < end.y || (start.y == end.y && start.x <= end.x) {
            (start, end)
        } else {
            (end, start)
        };

        let mut result = String::new();
        for y in start.y..=end.y {
             if let Some(row) = self.rows.get(y) {
                 let start_x = if y == start.y { start.x } else { 0 };
                 let end_x = if y == end.y { end.x.min(row.len()) } else { row.len() };
                 
                 if start_x < row.len() {
                      let s: String = row.content.chars().skip(start_x).take(end_x - start_x + 1).collect(); // Inclusive end
                      result.push_str(&s);
                 }
                 if y < end.y {
                     result.push('\n');
                 }
             }
        }
        result
    }

    pub fn delete_range(&mut self, start: &Position, end: &Position) {
         let (start, end) = if start.y < end.y || (start.y == end.y && start.x <= end.x) {
            (start, end)
        } else {
            (end, start)
        };
        
        // Naive implementation: delete char by char from end to start to avoid index shifts affecting earlier content (though we are deletingrange so it matters less if we start from end)
        // Better:
        // 1. Delete part of first row
        // 2. Remove intermediate rows
        // 3. Delete part of last row and merge with first
        
        // Simple approach using existing delete:
        // Moving cursor to 'end' is hard because text shifts.
        // But if we delete from 'start', all subsequent chars shift left.
        // So we can just repeatedly delete at 'start' until we reach 'end'.
        // BUT 'end' position becomes invalid after first delete.
        // So we need to calculate number of characters/lines to delete? No that's hard.
        
        // Let's rely on logic: We know exactly the range.
        if start.y == end.y {
            if let Some(row) = self.rows.get_mut(start.y) {
                // Delete from start.x to end.x inclusive
                for _ in start.x..=end.x {
                    row.delete(start.x);
                }
            }
        } else {
             // Multi-line delete
             // 1. Truncate start row
             if let Some(row) = self.rows.get_mut(start.y) {
                 // Keep 0..start.x
                 let keep: String = row.content.chars().take(start.x).collect();
                 row.content = keep;
                 row.len = start.x; // Update len manually or via method? Row::From rebuilds?
                                    // Row fields are private? No they are pub in my code? 
                                    // Oops Row len is private read but public in struct def?
                                    // Let's check Row struct... it is `pub content` and `len`
                 // Let's use delete() in a loop for safety or add truncate?
                 // Let's Re-read row.rs content? No I wrote it.
                 // Let's use just `delete` from right? 
                 // Actually, easy way:
                 // Construct new content for Start Row = StartRow[..start.x] + EndRow[end.x+1..]
                 // Delete all rows between start+1 and end (inclusive)
             }
             
             // Get content from end row
             let end_row_remainder = if let Some(row) = self.rows.get(end.y) {
                 row.content.chars().skip(end.x + 1).collect::<String>()
             } else {
                 String::new()
             };
             
             // Remove rows from start.y + 1 to end.y inclusive
             // We need to remove end.y - (start.y + 1) + 1 rows
             let num_to_remove = end.y - start.y;
             for _ in 0..num_to_remove {
                 if start.y + 1 < self.rows.len() {
                     self.rows.remove(start.y + 1);
                 }
             }
             
             // Append remainder to start row
             if let Some(row) = self.rows.get_mut(start.y) {
                 let remainder_len = end_row_remainder.chars().count();
                 row.content.push_str(&end_row_remainder);
                 row.len += remainder_len;
             }
        }
    }
}

use crate::editor::Position;
use crate::row::Row;
use std::fs;
use std::io::{Error, Write};

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

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

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if query.is_empty() {
            return None;
        }

        let mut y = at.y;
        
        // Forward Search
        if direction == SearchDirection::Forward {
            // 1. Search current line starting from at.x (+1 to avoid finding current char if it matches? usually find next means AFTER cursor)
            // But if cursor is ON the match, 'n' should go to next.
            // So we start searching from at.x + 1 usually?
            // If I type /foo, and I am at start of file, I want to find foo at line 1.
            // If I am ON foo, 'n' should find next foo.
            // Let's assume 'at' is current cursor.
            // We search from x. If match starts exactly at x, it counts as "found" (e.g. initial search).
            // But for 'n' (Next), the editor logic usually advances x before calling find, OR find takes an arg "skip_current".
            // Let's make find inclusive of 'at', and let Editor handle 'n' by passing at.x + 1.
            
            let mut start_x = at.x;
            
            for _ in 0..=self.len() {
                if let Some(row) = self.rows.get(y) {
                    // Find all occurrences in row
                    // We need char indices
                    let mut matches = Vec::new();
                    for (byte_idx, _) in row.content.match_indices(query) {
                        matches.push(row.content[..byte_idx].chars().count());
                    }
                    
                    // Find first match >= start_x
                    if let Some(&match_x) = matches.iter().find(|&&x| x >= start_x) {
                        return Some(Position { x: match_x, y });
                    }
                }
                
                y = (y + 1) % self.len();
                start_x = 0; // For next lines, start from 0
            }
        } else {
            // Backward Search
            // 1. Search current line up to at.x
            // logic: find max match_x such that match_x < at.x (strictly? usually yes for 'N' if we are on match)
            // Or inclusive?
            // For 'N', if we are on a match start, we want previous one.
            // Let's assume exclusive upper bound? 
            // Let's verify: Editor will pass 'at'.
            // If I am on match, 'N' should go back.
            // So we look for matches < at.x
            
            let mut limit_x = at.x; // We look for match_x < limit_x ?? 
            // Actually, simply: If I am at 10, and match is at 10.
            // N should go to occurrence before 10.
            // So we strictly filter < at.x check?
            
            // Wait, we iterate rows backwards.
            
            for _ in 0..=self.len() {
                if let Some(row) = self.rows.get(y) {
                     let mut matches = Vec::new();
                    for (byte_idx, _) in row.content.match_indices(query) {
                        matches.push(row.content[..byte_idx].chars().count());
                    }
                    
                    // Find last match < limit_x
                    // Use simple filter
                    // Note: if limit_x is None (conceptually infinity for other lines), we take max.
                    
                    // Logic:
                    // If this is the START line (first iteration): find match < at.x
                    // If this is NOT start line: find any match (max one)
                    
                    let candidate = if y == at.y && limit_x != usize::MAX {
                        matches.iter().filter(|&&x| x < at.x).last()
                    } else {
                        matches.last()
                    };
                    
                    if let Some(&match_x) = candidate {
                        return Some(Position { x: match_x, y });
                    }
                }
                
                if y == 0 {
                    y = self.len().saturating_sub(1);
                } else {
                    y -= 1;
                }
                limit_x = usize::MAX; // Reset limit for next lines
            }
        }

        None
    }
}

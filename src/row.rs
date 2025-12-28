#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighlightType {
    None,
    Number,
    Keyword,
    String,
    Comment,
}

#[derive(Default)]
pub struct Row {
    pub content: String,
    pub len: usize,
    pub highlighting: Vec<HighlightType>,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let content = String::from(slice);
        let len = content.chars().count();
        let highlighting = vec![HighlightType::None; len];
        Self {
            content,
            len,
            highlighting,
        }
    }
}

impl Row {
    pub fn len(&self) -> usize {
        self.len
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len {
            self.content.push(c);
            self.highlighting.push(HighlightType::None);
        } else {
            let mut result: String = self.content.chars().take(at).collect();
            result.push(c);
            let rest: String = self.content.chars().skip(at).collect();
            result.push_str(&rest);
            self.content = result;

            self.highlighting.insert(at, HighlightType::None);
        }
        self.len += 1;
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len {
            return;
        }
        let mut result: String = self.content.chars().take(at).collect();
        let rest: String = self.content.chars().skip(at + 1).collect();
        result.push_str(&rest);
        self.content = result;

        self.highlighting.remove(at);
        self.len -= 1;
    }

    pub fn append(&mut self, new: &Row) {
        self.content = format!("{}{}", self.content, new.content);
        self.highlighting.extend(&new.highlighting);
        self.len += new.len;
    }

    pub fn split(&mut self, at: usize) -> Row {
        let beginning: String = self.content.chars().take(at).collect();
        let remainder: String = self.content.chars().skip(at).collect();

        let highlighting_remainder = self.highlighting.split_off(at);

        self.content = beginning;
        self.len = at;

        Row {
            content: remainder,
            len: highlighting_remainder.len(),
            highlighting: highlighting_remainder,
        }
    }

    pub fn update_highlighting(&mut self, syntax: &crate::syntax::Syntax) {
        self.highlighting = vec![HighlightType::None; self.len];
        let chars: Vec<char> = self.content.chars().collect();
        let mut i = 0;
        let mut in_string = false;

        while i < chars.len() {
            let c = chars[i];

            if in_string {
                self.highlighting[i] = HighlightType::String;
                if c == '"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }

            if c == '"' {
                in_string = true;
                self.highlighting[i] = HighlightType::String;
                i += 1;
                continue;
            }

            // Comment
            let comment_start = syntax.single_line_comment;
            if !comment_start.is_empty()
                && i + comment_start.len() <= chars.len()
                && &self.content[i..i + comment_start.len()] == comment_start
            {
                while i < chars.len() {
                    self.highlighting[i] = HighlightType::Comment;
                    i += 1;
                }
                break;
            }

            if c.is_ascii_digit() {
                self.highlighting[i] = HighlightType::Number;
            }

            // Keyword detection
            let keywords = syntax.keywords;
            for &kw in keywords {
                let kw_chars: Vec<char> = kw.chars().collect();
                if i + kw_chars.len() <= chars.len() {
                    // Check match
                    let matches = chars[i..i + kw_chars.len()]
                        .iter()
                        .zip(kw_chars.iter())
                        .all(|(a, b)| a == b);
                    if matches {
                        // Check boundaries
                        let before_ok =
                            i == 0 || !chars[i - 1].is_alphanumeric() && chars[i - 1] != '_';
                        let after_ok = i + kw_chars.len() == chars.len()
                            || !chars[i + kw_chars.len()].is_alphanumeric()
                                && chars[i + kw_chars.len()] != '_';

                        if before_ok && after_ok {
                            for j in 0..kw_chars.len() {
                                self.highlighting[i + j] = HighlightType::Keyword;
                            }
                            // Don't advance immediately based on keyword length to avoid issues?
                            // Actually we should advance.
                            // But wait, the outer loop advances 1.
                            // If we match, we should skip
                            // But my loop is `while i < len`.
                            // So I can advance `i` here.

                            // NOTE: I cannot modify `i` and `continue` easily without affecting loop structure if I don't handle it carefully.
                            // Let's just break the keyword loop and let logic handle it?
                            // If I found a keyword, I highlighted it. I should assume I processed these chars.

                            // Let's store finding.
                            i += kw_chars.len() - 1; // -1 because loop does i++
                            // break inner loop
                            // wait, `continue` here continues `for` loop?
                            // I want to continue `while` loop.
                            // labeled break/continue?
                            // Rust supports loop labels.
                            // I'll use a boolean.
                            break; // break keyword loop
                        }
                    }
                }
            }

            i += 1;
        }
    }
}

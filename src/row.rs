#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighlightType {
    None,
    Number,
    Keyword,
    Type,
    ControlFlow,
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

            // Token detection (Keywords, Types, ControlFlow)
            let mut matched_token = false;
            for (tokens, highlight_type) in [
                (syntax.types, HighlightType::Type),
                (syntax.control_flow, HighlightType::ControlFlow),
                (syntax.keywords, HighlightType::Keyword),
            ] {
                for &token in tokens {
                    let token_chars: Vec<char> = token.chars().collect();
                    if i + token_chars.len() <= chars.len() {
                        let matches = chars[i..i + token_chars.len()]
                            .iter()
                            .zip(token_chars.iter())
                            .all(|(a, b)| a == b);

                        if matches {
                            // Check boundaries (must be separated by non-alphanumeric, except '_')
                            let before_ok =
                                i == 0 || !chars[i - 1].is_alphanumeric() && chars[i - 1] != '_';
                            let after_ok = i + token_chars.len() == chars.len()
                                || !chars[i + token_chars.len()].is_alphanumeric()
                                    && chars[i + token_chars.len()] != '_';

                            if before_ok && after_ok {
                                for j in 0..token_chars.len() {
                                    self.highlighting[i + j] = highlight_type;
                                }
                                i += token_chars.len() - 1;
                                matched_token = true;
                                break;
                            }
                        }
                    }
                }
                if matched_token {
                    break;
                }
            }

            i += 1;
        }
    }
}

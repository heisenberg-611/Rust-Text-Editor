

#[derive(Default)]
pub struct Row {
    pub content: String,
    pub len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let content = String::from(slice);
        let len = content.chars().count();
        Self { content, len }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = std::cmp::min(end, self.content.len());
        let start = std::cmp::min(start, end);
        self.content.get(start..end).unwrap_or("").to_string()
    }

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
        } else {
            let mut result: String = self.content.chars().take(at).collect();
            result.push(c);
            let rest: String = self.content.chars().skip(at).collect();
            result.push_str(&rest);
            self.content = result;
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
        self.len -= 1;
    }

    pub fn append(&mut self, new: &Row) {
        self.content = format!("{}{}", self.content, new.content);
        self.len += new.len;
    }

    pub fn split(&mut self, at: usize) -> Row {
        let beginning: String = self.content.chars().take(at).collect();
        let remainder: String = self.content.chars().skip(at).collect();
        self.content = beginning;
        self.len = at;
        Row::from(remainder.as_str())
    }
}

pub struct Buffer {
    pub lines: Vec<String>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            lines: vec!["Hello, World!".to_string()],
        }
    }
}

impl Buffer {
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_buffer_has_hello_world() {
        let buffer = Buffer::default();
        assert_eq!(buffer.lines.len(), 1);
        assert_eq!(buffer.lines[0], "Hello, World!");
    }

    #[test]
    fn default_buffer_is_not_empty() {
        let buffer = Buffer::default();
        assert!(!buffer.is_empty());
    }

    #[test]
    fn empty_buffer() {
        let buffer = Buffer {
            lines: Vec::new(),
        };
        assert!(buffer.is_empty());
    }

    #[test]
    fn buffer_multiple_lines() {
        let buffer = Buffer {
            lines: vec![
                "line 1".to_string(),
                "line 2".to_string(),
                "line 3".to_string(),
            ],
        };
        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[1], "line 2");
    }
}

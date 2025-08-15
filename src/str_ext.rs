
pub trait Substr<'s> {
    fn substr(&'s self, start: usize, len: usize) -> Option<&'s str>;
}

impl<'s> Substr<'s> for &'s str {
    fn substr(&'s self, start: usize, len: usize) -> Option<&'s str> {
        if len == 0 {
            return None;
        }
        if start + len > self.len() {
            return None;
        }
        Some(&self[start..(start + len)])
    }
}

pub trait Textwise {
    fn pos(&self, n: usize) -> (usize, usize);
    fn line(&self, n: usize) -> &str;
}

impl Textwise for &str {
    fn pos(&self, n: usize) -> (usize, usize) {
        if n >= self.chars().count() {
            return (0, 0);
        }

        let mut line = 1;
        let mut col = 0;

        for (idx, c) in self.char_indices() {
            if idx == n {
                return (line, col);
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        unreachable!()
    }

    fn line(&self, line_n: usize) -> &str {
        if line_n == 0 {
            panic!("Line number must be greater than zero.");
        }
        let line_n = line_n - 1;
        for (idx, line) in self.lines().enumerate() {
            if idx == line_n {
                return line;
            }
        }

        panic!("Index `{}` exceeds the amount of lines.", line_n + 1);
    }
}


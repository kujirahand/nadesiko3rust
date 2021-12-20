use crate::charutils;

pub struct StrCur {
    pub src: Vec<char>,
    pub index: usize,
    pub length: usize,
}

impl StrCur {
    pub fn from(src: &str) -> Self {
        let vc:Vec<char> = src.chars().collect();
        let len = vc.len();
        Self {
            src: vc,
            index: 0,
            length: len,
        }
    }
    pub fn peek(&self) -> char {
        if self.can_read() {
            return self.src[self.index];
        }
        '\0'
    }
    pub fn next(&mut self) -> char {
        if self.can_read() {
            let ch = self.src[self.index];
            self.index += 1;
            return ch;
        }
        '\0'
    }
    pub fn seek(&mut self, inc_value: i32) {
        if inc_value > 0 {
            self.index += inc_value as usize;
            if self.index >= self.length { self.index = self.length }
        } else {
            let iv = inc_value.abs() as usize;
            if self.index < iv {
                self.index = 0;
            } else {
                self.index -= iv;
            }
        }
    }
    pub fn peek_half(&self) -> char {
        let ch = self.peek();
        charutils::to_half_ascii(ch)
    }
    pub fn can_read(&self) -> bool {
        self.index < self.length
    }
    pub fn eq_str(&self, target: &str) -> bool {
        // 文字列の長さが異なる
        if self.index + target.len() >= self.length {
            return false;
        }
        // 一つずつ比較
        for (i, c) in target.chars().enumerate() {
            if self.src[i + self.index] != c {
                return false;
            }
        }
        true
    }
    pub fn skip_space(&mut self) {
        loop {
            let c = self.peek_half();
            if c == ' ' || c == '\t' {
                self.next();
                continue;
            }
            break;
        }
    }
    pub fn get_token(&mut self, delimiter: char) -> Vec<char> {
        let mut result: Vec<char> = vec![];
        while self.can_read() {
            let ch = self.next();
            if ch == delimiter {
                return result;
            }
            result.push(ch);
        }
        result
    }
    pub fn get_token_tostr(&mut self, delimiter: char) -> String {
        self.get_token(delimiter).iter().collect()
    }
}

#[cfg(test)]
mod test_prepare {
    use super::*;
    #[test]
    fn strcur_test() {
        // 1
        let mut cur = StrCur::from("a//b");
        assert_eq!(cur.eq_str("a//"), true);
        assert_eq!(cur.eq_str("ab"), false);
        let ch = cur.next();
        assert_eq!(ch, 'a');
        assert_eq!(cur.eq_str("//"), true);
        // skip_space
        let mut cur = StrCur::from("a   b");
        assert_eq!(cur.next(), 'a');
        cur.skip_space();
        assert_eq!(cur.next(), 'b');        
        assert_eq!(cur.can_read(), false);
        // seek
        let mut cur = StrCur::from("012345");
        cur.seek(3);
        assert_eq!(cur.peek(), '3');
        cur.seek(-30);
        assert_eq!(cur.peek(), '0');
        cur.seek(30);
        assert_eq!(cur.can_read(), false);
        // get_token
        let mut cur = StrCur::from("aaa,bbb,ccc");
        assert_eq!(cur.get_token_tostr(','), "aaa");
        assert_eq!(cur.get_token_tostr(','), "bbb");
        assert_eq!(cur.get_token_tostr(','), "ccc");
        assert_eq!(cur.can_read(), false);
    }
}
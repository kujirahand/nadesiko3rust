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
        if self.index + target.len() > self.length {
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
    pub fn get_str(&mut self, length: usize) -> String {
        let mut result: Vec<char> = vec![];
        let mut remain = length;
        while self.can_read() {
            if remain == 0 { break; }
            result.push(self.next());
            remain -= 1;
        }
        result.iter().collect()
    }
    pub fn skip_space(&mut self) -> bool {
        let mut changed = false;
        loop {
            let c = self.peek_half();
            if c == ' ' || c == '\t' {
                self.next();
                changed = true;
                continue;
            }
            break;
        }
        changed
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
    pub fn get_token_str(&mut self, delimiter: &str) -> Vec<char> {
        let mut result: Vec<char> = vec![];
        while self.can_read() {
            if self.eq_str(delimiter) {
                self.index += delimiter.len();
                return result;
            }
            result.push(self.next());
        }
        result
    }
    pub fn get_token_tostr(&mut self, delimiter: char) -> String {
        self.get_token(delimiter).iter().collect()
    }
    pub fn get_token_str_tostr(&mut self, delimiter: &str) -> String {
        self.get_token_str(delimiter).iter().collect()
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
    }
    #[test]
    fn get_token_test () {
        // get_token
        let mut cur = StrCur::from("aaa,bbb,ccc");
        assert_eq!(cur.get_token_tostr(','), "aaa");
        assert_eq!(cur.get_token_tostr(','), "bbb");
        assert_eq!(cur.get_token_tostr(','), "ccc");
        assert_eq!(cur.can_read(), false);
    }
    #[test]
    fn eq_str_test () {
        let mut cur = StrCur::from("aaa/*bbb*/ccc");
        assert_eq!(cur.eq_str("aaa"), true);
        cur.seek(3);  
        assert_eq!(cur.eq_str("/*"), true);
        cur.seek(2);  
        assert_eq!(cur.eq_str("bbb"), true);
        cur.seek(3);
        assert_eq!(cur.eq_str("*/"), true);
        cur.seek(2);
        assert_eq!(cur.eq_str("ccc"), true);
    }
    #[test]
    fn get_token_str_test () {
        // get_token
        let mut cur = StrCur::from("aaa::bbb::ccc");
        assert_eq!(cur.get_token_str_tostr("::"), "aaa");
        assert_eq!(cur.get_token_str_tostr("::"), "bbb");
        assert_eq!(cur.get_token_str_tostr("::"), "ccc");
        assert_eq!(cur.can_read(), false);
        //
        let mut cur = StrCur::from("/*AAA*/BBB");
        assert_eq!(cur.get_token_str_tostr("/*"), "");
        assert_eq!(cur.get_token_str_tostr("*/"), "AAA");
        assert_eq!(cur.get_token_str_tostr("*/"), "BBB");
        assert_eq!(cur.can_read(), false);
        //
        let mut cur = StrCur::from("//abc\n\n/*fff*/");
        assert_eq!(cur.get_token_str_tostr("/*"), "//abc\n\n");
        assert_eq!(cur.get_token_str_tostr("*/"), "fff");
        assert_eq!(cur.can_read(), false);
    }
}
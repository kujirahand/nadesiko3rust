//! 文字列をカーソルで操作する

use crate::kanautils;

/// 文字列カーソル
pub struct StrCur {
    pub src: Vec<char>,
    length: usize, // private : user can not change
    index: usize,
    top_index: usize, // カーソルの途中からインデックスを取得するためのもの
}

#[allow(dead_code)]
impl StrCur {
    pub fn from(src: &str) -> Self {
        Self::from_source(src, 0)
    }
    pub fn from_source(src: &str, top_index: usize) -> Self {
        let vc:Vec<char> = src.chars().collect();
        let len = vc.len();
        Self {
            src: vc,
            index: 0,
            length: len,
            top_index,
        }
    }
    pub fn peek(&self) -> char {
        if !self.can_read() { return '\0'; }
        return self.src[self.index];
    }
    pub fn peek_in_range(&self, min: char, max: char) -> bool {
        if !self.can_read() { return false; }
        let ch = self.peek();
        return (min <= ch) && (ch <= max)
    }
    pub fn next(&mut self) -> char {
        if !self.can_read() { return '\0'; }
        let ch = self.src[self.index];
        self.index += 1;
        return ch;
    }
    pub fn seek(&mut self, inc_value: i64) {
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
    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }
    pub fn get_index(&mut self) -> usize {
        self.index + self.top_index
    }
    pub fn get_index_i(&mut self) -> i64 {
        (self.index + self.top_index) as i64
    }
    pub fn peek_half(&self) -> char {
        let ch = self.peek();
        kanautils::to_half_ascii(ch)
    }
    pub fn can_read(&self) -> bool {
        self.index < self.length
    }
    pub fn eq_str(&self, target: &str) -> bool {
        // 一つずつ比較
        for (i, c) in target.chars().enumerate() {
            let index = i + self.index;
            if index >= self.length { return false; }
            if self.src[index] != c { return false; }
        }
        true
    }
    pub fn peek_chars(&self, length: usize) -> Vec<char> {
        let mut result: Vec<char> = vec![];
        for i in 0..length {
            let idx = i + self.index;
            if idx >= self.length { break; }
            let ch = self.src[idx];
            result.push(ch);
        }
        result
    }
    pub fn peek_str(&self, length: usize) -> String {
        let result = self.peek_chars(length);
        result.iter().collect()
    }
    pub fn get_str(&mut self, length: usize) -> String {
        let result = self.peek_chars(length);
        self.seek(result.len() as i64);
        result.iter().collect()
    }
    pub fn get_range(&mut self, min:char, max:char) -> Vec<char> {
        let mut result: Vec<char> = vec![];
        while self.can_read() {
            let ch = self.peek();
            if min <= ch && ch <= max {
                result.push(self.next());
                continue;
            }
            break;
        }
        result
    }
    pub fn get_range_str(&mut self, min:char, max:char) -> String {
        let s = self.get_range(min, max);
        s.iter().collect()
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
        //
        let cur = StrCur::from("あいうえお");
        assert_eq!(cur.eq_str("あいうえお"), true);
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
    #[test]
    fn get_str_test () {
        // get_str
        let mut cur = StrCur::from("aaa->bbb->ccc");
        assert_eq!(cur.get_str(3), "aaa");
        assert_eq!(cur.get_str(2), "->");
        assert_eq!(cur.get_str(3), "bbb");
        assert_eq!(cur.get_str(2), "->");
        assert_eq!(cur.get_str(5), "ccc");
    }
    #[test]
    fn get_range_test() {
        let mut cur = StrCur::from("123abc456ccc");
        assert_eq!(cur.get_range_str('0','9'), "123");
        assert_eq!(cur.get_range_str('a','z'), "abc");
        assert_eq!(cur.get_range_str('0','9'), "456");
        assert_eq!(cur.get_range_str('a','z'), "ccc");
    }
    #[test]
    fn next_test2() {
        let mut cur = StrCur::from("123");
        assert_eq!(cur.next(), '1');
        assert_eq!(cur.next(), '2');
        assert_eq!(cur.next(), '3');
        assert_eq!(cur.next(), '\0');
        assert_eq!(cur.next(), '\0');
    }
}
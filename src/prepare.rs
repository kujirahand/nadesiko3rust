#[allow(dead_code)]

use crate::charutils;

struct StrCur {
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
}

pub fn convert(src: &str) -> String {
    let mut result = String::new();
    let mut cur = StrCur::from(src);
    let mut is_str = false;
    let mut end_of_str = 0 as char;
    let mut is_line_comment = false;
    let mut is_range_comment = false;
    while cur.can_read() {
        let ch = cur.next();
        let ch_half = charutils::to_half_ascii(ch);
        // string
        if is_str {
            if ch == end_of_str {
                is_str = false;
                result.push(ch);
                continue;
            }
            result.push(ch);
            continue;
        }
        // comment
        if is_line_comment {
            if ch == '\n' {
                is_line_comment = false;
            }
            result.push(ch);
            continue;
        }
        if is_range_comment {
            if ch_half == '*' {
                let ch2 = cur.peek_half();
                if ch2 == '/' {
                    is_range_comment = false;
                    result.push_str("*/");
                    cur.next();
                    continue;
                }
            }
            result.push(ch);
            continue;
        }

        // check string
        if ch == '"' || ch == '\'' {
            is_str = true;
            end_of_str = ch;
            result.push(ch);
            continue;
        }
        if ch == '「' {
            is_str = true;
            end_of_str = '」';
            result.push(ch);
            continue;
        }
        if ch == '『' {
            is_str = true;
            end_of_str = '』';
            result.push(ch);
            continue;
        }
        // check comment
        if ch_half == '/' {
            let ch2 = cur.peek_half();
            if ch2 == '/' {
                is_line_comment = true;
                result.push_str("//");
                cur.next();
                continue;
            }
            if ch2 == '*' {
                is_range_comment = true;
                result.push_str("/*");
                cur.next();
                continue;
            }
        }
        // others
        result.push(ch_half);
    }
    return result;
}

#[cfg(test)]
mod test_prepare {
    use super::*;
    #[test]
    fn convert_test() {
        let s = convert("");
        assert_eq!(s, String::from(""));
        let s = convert("abc");
        assert_eq!(s, String::from("abc"));
        let s = convert("！！/*！！*/！！");
        assert_eq!(s, String::from("!!/*！！*/!!"));
    }
}
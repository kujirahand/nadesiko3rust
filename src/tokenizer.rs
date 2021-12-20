use crate::prepare;
use crate::strcur::StrCur;

#[derive(Debug)]
pub struct TokenInfo {
    pub label: String,
    pub josi: String,
    pub line: u32,
}

impl TokenInfo {
    pub fn new(label: String, josi: String, line: u32) -> Self {
        Self {label: label, josi: josi, line: line}
    }
    pub fn new_label(label: String, line: u32) -> Self {
        Self {label: label, josi: String::new(), line: line}
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Token {
    Comment(TokenInfo),
    Eol(TokenInfo),
    Number(TokenInfo),
    String(TokenInfo),
    StringEx(TokenInfo),
    Word(TokenInfo),
    Operator(TokenInfo),
}

pub fn tokenize(src: &str) -> Vec<Token> {
    let src = prepare::convert(src);
    let mut cur = StrCur::from(&src);
    let mut result: Vec<Token> = vec![];
    let mut line = 0;
    while cur.can_read() {
        cur.skip_space();
        if cur.eq_str("//") {
            let rem = cur.get_token_tostr('\n');
            let tok = Token::Comment(TokenInfo::new_label(rem, line));
            result.push(tok);
            line += 1;
            continue;
        }
        cur.next();
    }
    result
}

pub fn tokens_string(vt: &Vec<Token>) -> String {
    let mut res = String::new();
    for tok in vt.iter() {
        let s: String = match tok {
            Token::Comment(t) => format!("Comment:{}", t.label),
            _ => format!("{:?}",tok),
        };
        let s = format!("[{}],", s);
        res.push_str(&s);
    }
    format!("[{}]", res)
}

#[cfg(test)]
mod test_tokenizer {
    use super::*;
    #[test]
    fn test_tokenize() {
        let t = tokenize("//abc");
        assert_eq!(tokens_string(&t), "");
    }
}
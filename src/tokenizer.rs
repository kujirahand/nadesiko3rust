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
    pub fn new_label_str(label: &str, line: u32) -> Self {
        Self {label: String::from(label), josi: String::new(), line: line}
    }
    pub fn new_label_char(label: char, line: u32) -> Self {
        Self {label: String::from(label), josi: String::new(), line: line}
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
    Flag(TokenInfo),
}

pub fn tokenize(src: &str) -> Vec<Token> {
    let src = prepare::convert(src);
    let mut cur = StrCur::from(&src);
    let mut result: Vec<Token> = vec![];
    let mut line = 0;
    while cur.can_read() {
        if cur.skip_space() { continue; }
        let ch = cur.peek_half();
        if ch == '\n' {
            let lf = cur.next();
            let tok = Token::Eol(TokenInfo::new_label_char(lf, line));
            result.push(tok);
            line += 1;
            continue;
        }
        if ch == '/' {
            // line comment
            if cur.eq_str("//") {
                cur.seek(2); // skip "//"
                let rem = cur.get_token_tostr('\n');
                let tok = Token::Comment(TokenInfo::new_label(rem, line));
                result.push(tok);
                line += 1;
                continue;
            }
            // range comment
            if cur.eq_str("/*") {
                cur.seek(2); // skio "/*"
                let rem = cur.get_token_str("*/");
                println!("@@@{}@@@", rem.iter().collect::<String>());
                let mut ret_cnt = 0;
                for c in rem.iter() {
                    if *c == '\n' { ret_cnt += 1; }
                }
                let rem_s = rem.iter().collect();
                let tok = Token::Comment(TokenInfo::new_label(rem_s, line));
                result.push(tok);
                line += ret_cnt;
                continue;
            }
            // flag
            let flag = cur.next();
            let tok = Token::Flag(TokenInfo::new_label_char(flag, line));
            result.push(tok);
            continue;
        }
        // number
        if (ch >= '0') && (ch <= '9') {
            
        }
        
        // pass
        println!("pass:{}", ch);
        cur.next();
    }
    result
}

pub fn tokens_string(vt: &Vec<Token>) -> String {
    let mut res = String::new();
    for tok in vt.iter() {
        let s: String = match tok {
            Token::Comment(t) => format!("Comment:{}", t.label),
            Token::Eol(_) => format!("Eol"),
            Token::Number(t) => format!("Number:{}", t.label),
            Token::String(t) => format!("String:{}", t.label),
            Token::StringEx(t) => format!("StringEx:{}", t.label),
            Token::Word(t) => format!("Word:{}", t.label),
            Token::Flag(t) => format!("Flag:{}", t.label),
            _ => format!("{:?}",tok),
        };
        let s = format!("[{}]", s);
        res.push_str(&s);
    }
    format!("{}", res)
}

#[cfg(test)]
mod test_tokenizer {
    use super::*;
    #[test]
    fn test_tokenize() {
        let t = tokenize("//abc");
        assert_eq!(tokens_string(&t), "[Comment:abc]");
        //let t = tokenize("//abc\n\n/*ABC*/");
        //assert_eq!(tokens_string(&t), "[Comment:abc][Eol][Comment:ABC]");
    }
}

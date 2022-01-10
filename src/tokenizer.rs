use crate::prepare;
use crate::strcur::StrCur;
use crate::charutils;
use crate::josi;

#[derive(Debug)]
pub struct TokenInfo {
    pub label: String,
    pub josi: Option<String>,
    pub line: u32,
}
impl TokenInfo {
    pub fn new(label: String, josi: Option<String>, line: u32) -> Self {
        Self {label: label, josi: josi, line: line}
    }
    pub fn new_label(label: String, line: u32) -> Self {
        Self {label: label, josi: None, line: line}
    }
    pub fn new_label_str(label: &str, line: u32) -> Self {
        Self {label: String::from(label), josi: None, line: line}
    }
    pub fn new_label_char(label: char, line: u32) -> Self {
        Self {label: String::from(label), josi: None, line: line}
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Token {
    Comment(TokenInfo),
    Eol(TokenInfo),
    Int(TokenInfo, i64),
    Number(TokenInfo, f64),
    String(TokenInfo),
    StringEx(TokenInfo),
    Word(TokenInfo),
    Flag(TokenInfo),
    ParenL(TokenInfo),
    ParenR(TokenInfo),
    BracketL(TokenInfo),
    BracketR(TokenInfo),
    CurBracketL(TokenInfo),
    CurBracketR(TokenInfo),
}
impl std::fmt::Display for Token {

    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // 
        let get_value = |t: &TokenInfo| -> String {
            match &t.josi {
                Some(j) => { format!("{}/{}", t.label, j) },
                None    => { format!("{}", t.label) },
            }
        };
        match self {
            Token::Comment(t) => write!(f, "Comment:{}", get_value(t)),
            Token::Eol(_) => write!(f, "Eol"),
            Token::Int(t, _) => write!(f, "Int:{}", get_value(t)),
            Token::Number(t, _) => write!(f, "Number:{}", get_value(t)),
            Token::String(t) => write!(f, "String:{}", get_value(t)),
            Token::StringEx(t) => write!(f, "StringEx:{}", get_value(t)),
            Token::Word(t) => write!(f, "Word:{}", get_value(t)),
            Token::Flag(t) => write!(f, "Flag:{}", get_value(t)),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub fn tokens_string(vt: &Vec<Token>) -> String {
    let mut res = String::new();
    for tok in vt.iter() {
        let s = format!("[{}]", tok);
        res.push_str(&s);
    }
    format!("{}", res)
}

pub fn tokenize(src: &str) -> Vec<Token> {
    let src = prepare::convert(src);
    let mut cur = StrCur::from(&src);
    let mut result: Vec<Token> = vec![];
    let mut line = 0;
    while cur.can_read() {
        if cur.skip_space() { continue; }
        let ch = cur.peek();
        match ch {
            '\n' => { result.push(read_lf(&mut cur, &mut line)); continue; },
            '/' => { result.push(read_slash(&mut cur, &mut line)); continue; },
            '(' => { result.push(Token::ParenL(TokenInfo::new_label_char(cur.next(), line))); continue; },
            ')' => { result.push(Token::ParenR(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '[' => { result.push(Token::BracketL(TokenInfo::new_label_char(cur.next(), line))); continue; },
            ']' => { result.push(Token::BracketR(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '\\' => { result.push(Token::Flag(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '^' => { result.push(Token::Flag(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '`' => { result.push(Token::Flag(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '{' => { result.push(Token::CurBracketL(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '}' => { result.push(Token::CurBracketR(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '|' => { result.push(Token::Flag(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '~' => { result.push(Token::Flag(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '!'..='.' => { result.push(Token::Flag(TokenInfo::new_label_char(cur.next(), line))); continue; },
            ':'..='@' => { result.push(Token::Flag(TokenInfo::new_label_char(cur.next(), line))); continue; },
            '0'..='9' => { result.push(read_number(&mut cur, &mut line)); continue; },
            'a'..='z' | 'A'..='Z' | '_' => { result.push(read_word(&mut cur, &mut line)); continue; }
            n if n > (0xE0 as char) => { result.push(read_word(&mut cur, &mut line)); continue; }
            _ => {}
        }
        // pass
        println!("pass:{}", ch);
        cur.next();
    }
    result
}

fn read_lf(cur: &mut StrCur, line: &mut u32) -> Token {
    let lf = cur.next();
    let tok = Token::Eol(TokenInfo::new_label_char(lf, *line));
    *line += 1;
    tok
}

fn read_slash(cur: &mut StrCur, line: &mut u32) -> Token {
    // line comment
    if cur.eq_str("//") {
        cur.seek(2); // skip "//"
        let rem = cur.get_token_tostr('\n');
        let tok = Token::Comment(TokenInfo::new_label(rem, *line));
        *line += 1;
        return tok;
    }
    // range comment
    if cur.eq_str("/*") {
        cur.seek(2); // skio "/*"
        let rem = cur.get_token_str("*/");
        let mut ret_cnt = 0;
        for c in rem.iter() {
            if *c == '\n' { ret_cnt += 1; }
        }
        let rem_s = rem.iter().collect();
        let tok = Token::Comment(TokenInfo::new_label(rem_s, *line));
        *line += ret_cnt;
        return tok;
    }
    // flag
    let flag = cur.next();
    let tok = Token::Flag(TokenInfo::new_label_char(flag, *line));
    tok
}

fn read_number(cur: &mut StrCur, line: &mut u32) -> Token {
    let mut vc: Vec<char> = vec![];
    while cur.peek_in_range('0', '9') {
        vc.push(cur.next());
    }
    if cur.peek() == '.' {
        vc.push(cur.next());
        while cur.peek_in_range('0', '9') {
            vc.push(cur.next());
        }
        let num_s: String = vc.iter().collect();
        let num_f: f64 = num_s.parse().unwrap_or(0.0);
        let josi_opt = josi::read_josi(cur);
        return Token::Number(TokenInfo::new(num_s, josi_opt, *line), num_f);
    }
    let num_s: String = vc.iter().collect();
    let num_i: i64 = num_s.parse().unwrap_or(0);
    let josi_opt = josi::read_josi(cur);
    return Token::Int(TokenInfo::new(num_s, josi_opt, *line), num_i);
}

fn read_word(cur: &mut StrCur, line: &mut u32) -> Token {
    let mut word: Vec<char> = vec![];
    let mut josi_opt:Option<String> = None;
    
    // ひらがなスタートなら1文字目は助詞にならない
    if charutils::is_hiragana(cur.peek()) {
        word.push(cur.next());
    }
    
    while cur.can_read() {
        let c = cur.peek();
        // 助詞か？
        if charutils::is_hiragana(c) {
            josi_opt = josi::read_josi(cur);
            match josi_opt {
                Some(_) => break, // 助詞なら繰り返しを抜ける
                None => {}, // pass
            }
        }
        // wordになり得る文字か？
        if charutils::is_word_chars(c) {
            word.push(cur.next());
            println!("read:{}", c);
            continue;
        }
        break;
    }
    let word_s = word.iter().collect();
    Token::Word(TokenInfo::new(word_s, josi_opt, *line))
}


#[cfg(test)]
mod test_tokenizer {
    use super::*;
    #[test]
    fn test_tokenize() {
        let t = tokenize("//abc");
        assert_eq!(tokens_string(&t), "[Comment:abc]");
        let t = tokenize("//abc\n\n/*ABC*/");
        assert_eq!(tokens_string(&t), "[Comment:abc][Eol][Comment:ABC]");
        let t = tokenize("3\n3.14");
        assert_eq!(tokens_string(&t), "[Int:3][Eol][Number:3.14]");
        let t = tokenize("hoge=35");
        assert_eq!(tokens_string(&t), "[Word:hoge][Flag:=][Int:35]");
        let t = tokenize("年齢=15");
        assert_eq!(tokens_string(&t), "[Word:年齢][Flag:=][Int:15]");
    }
    #[test]
    fn test_tokenize_josi() {
        let t = tokenize("AからBまで");
        assert_eq!(tokens_string(&t), "[Word:A/から][Word:B/まで]");
        let t = tokenize("犬をネコへ");
        assert_eq!(tokens_string(&t), "[Word:犬/を][Word:ネコ/へ]");
    }
}

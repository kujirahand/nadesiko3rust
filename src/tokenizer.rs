use crate::prepare;
use crate::strcur::StrCur;
use crate::charutils;
use crate::josi_list;
use crate::reserve_word;

#[derive(Debug,Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub label: String,
    pub josi: Option<String>,
    pub line: u32,
}
impl Token {
    pub fn new(kind: TokenKind, label: String, josi: Option<String>, line: u32) -> Self {
        Self { kind, label, josi, line }
    }
    pub fn new_char(kind: TokenKind, label: char, line: u32) -> Self {
        Self {
            kind,
            label: String::from(label),
            josi: None,
            line,
        }
    }
    pub fn new_str(kind: TokenKind, label: &str, line: u32) -> Self {
        Self {
            kind,
            label: String::from(label),
            josi: None,
            line,
        }
    }
}
#[derive(Debug,Clone,PartialEq,Copy)]
pub enum TokenKind {
    None,
    Comment,
    Eol,
    Int,
    Number,
    String,
    StringEx,
    Word,
    Flag,
    ParenL,
    ParenR,
    BracketL,
    BracketR,
    CurBracketL,
    CurBracketR,
    If,
    Repeat,
}
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // 助詞の有無に生じて出力方式を変更する
        let get_value = |t: &Token| -> String {
            match &t.josi {
                Some(j) => { format!("{}/{}", t.label, j) },
                None    => { format!("{}", t.label) },
            }
        };
        let t = &self;
        match self.kind {
            TokenKind::None => write!(f, "None"),
            TokenKind::Comment => write!(f, "Comment:{}", get_value(t)),
            TokenKind::Eol => write!(f, "Eol"),
            TokenKind::Int => write!(f, "Int:{}", get_value(t)),
            TokenKind::Number => write!(f, "Number:{}", get_value(t)),
            TokenKind::String => write!(f, "String:{}", get_value(t)),
            TokenKind::StringEx => write!(f, "StringEx:{}", get_value(t)),
            TokenKind::Word => write!(f, "Word:{}", get_value(t)),
            TokenKind::Flag => write!(f, "Flag:{}", get_value(t)),
            TokenKind::ParenL => write!(f, "ParenL:{}", get_value(t)),
            TokenKind::ParenR => write!(f, "ParenR:{}", get_value(t)),
            TokenKind::If => write!(f, "If"),
            TokenKind::Repeat => write!(f, "Repeat"),
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

// tokenizeメソッドで使うマクロ手軽にトークンを生成する
macro_rules! flag_push {
    ( $type:expr, $result:expr, $cur: expr, $line: expr ) => {
        let tok = Token {
            kind: $type,
            label: String::from($cur.next()),
            josi: None,
            line: $line,
        };
        $result.push(tok);
    };
}
macro_rules! flag_push_josi {
    ( $type:expr, $result:expr, $cur: expr, $line: expr ) => {
        let label = String::from($cur.next());
        let josi_opt = josi_list::read_josi(&mut $cur);
        let tok = Token {
            kind: $type,
            label,
            josi: josi_opt,
            line: $line,
        };
        $result.push(tok);
    };
}

// 文字列をトークンに区切る
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
            '「' => { result.push(read_string(&mut cur, &mut line, '」', true)); continue; }
            '『' => { result.push(read_string(&mut cur, &mut line, '』', false)); continue; }
            '"' => { result.push(read_string(&mut cur, &mut line, '"', true)); continue; }
            '\'' => { result.push(read_string(&mut cur, &mut line, '\'', false)); continue; }
            '(' => { flag_push!(TokenKind::ParenL, result, cur, line); continue; },
            ')' => { flag_push_josi!(TokenKind::ParenR, result, cur, line); continue; },
            '[' => { flag_push!(TokenKind::BracketL, result, cur, line); continue; },
            ']' => { flag_push_josi!(TokenKind::BracketR, result, cur, line); continue; },
            '\\' => { flag_push!(TokenKind::Flag, result, cur, line); continue; },
            '^' => { flag_push!(TokenKind::Flag, result, cur, line); continue; },
            '`' => { flag_push!(TokenKind::Flag, result, cur, line); continue; },
            '{' => { flag_push!(TokenKind::CurBracketL, result, cur, line); continue; },
            '}' => { flag_push_josi!(TokenKind::CurBracketR, result, cur, line); continue; },
            '|' => { flag_push!(TokenKind::Flag, result, cur, line); continue; },
            '~' => { flag_push!(TokenKind::Flag, result, cur, line); continue; },
            '!'..='.' => { flag_push!(TokenKind::Flag, result, cur, line); continue; },
            ':'..='@' => { flag_push!(TokenKind::Flag, result, cur, line); continue; },
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
    let t = Token::new_char(TokenKind::Eol, lf, *line);
    *line += 1;
    return t;
}

fn read_slash(cur: &mut StrCur, line: &mut u32) -> Token {
    // line comment
    if cur.eq_str("//") {
        cur.seek(2); // skip "//"
        let rem = cur.get_token_tostr('\n');
        let tok = Token::new_str(TokenKind::Comment, &rem, *line);
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
        let rem_s: String = rem.iter().collect();
        let tok = Token::new_str(TokenKind::Comment, &rem_s, *line);
        *line += ret_cnt;
        return tok;
    }
    // flag
    let flag = cur.next();
    return Token::new_char(TokenKind::Flag, flag, *line);
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
        let josi_opt = josi_list::read_josi(cur);
        return Token::new(TokenKind::Number, num_s, josi_opt, *line);
    }
    let num_s: String = vc.iter().collect();
    let josi_opt = josi_list::read_josi(cur);
    return Token::new(TokenKind::Int, num_s, josi_opt, *line);
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
            josi_opt = josi_list::read_josi(cur);
            match josi_opt {
                Some(_) => break, // 助詞なら繰り返しを抜ける
                None => {}, // pass
            }
        }
        // wordになり得る文字か？
        if charutils::is_word_chars(c) {
            word.push(cur.next());
            continue;
        }
        break;
    }
    
    // 送りがなをカット
    word = delete_okurigana(word);
    let word_s: String = word.iter().collect();
    let kind = reserve_word::check_kind(&word_s);
    Token::new(kind, word_s, josi_opt, *line)
}

fn delete_okurigana(word: Vec<char>) -> Vec<char> {
    // 1文字なら送りがなはない
    if word.len() <= 1 {
        return word;
    }
    // (ex) 置き換える → 置換 ... 送りがなは漢字を挟んでも削る
    // (ex) お兄さん → お兄 ... 漢字の後ろのひらがなのみ削る
    // (ex) うたう → うたう ... 全部ひらがなであれば削らない
    // (ex) INTする → INT ... アルファベットも漢字と見なす
    let mut result: Vec<char> = vec![];
    let mut is_hajime_hiragana = true;
    for c in word.iter() {
        // 漢字?
        if !charutils::is_hiragana(*c) {
            is_hajime_hiragana = false;
            result.push(*c);
            continue;
        }
        // 冒頭のひらがなは追加し続ける
        if is_hajime_hiragana {
            result.push(*c);
            continue;
        }
    }
    result
}

fn delete_okurigana_str(word: &str) -> String {
    let word_v:Vec<char> = word.chars().collect();
    let res_v = delete_okurigana(word_v);
    res_v.iter().collect()
}

fn read_string(cur: &mut StrCur, line: &mut u32, end_flag: char, ex_str: bool) -> Token {
    cur.next(); // begin_flag
    let mut result: Vec<char> = vec![];
    let line_begin = *line;
    while cur.can_read() {
        let c = cur.next();
        if c == end_flag {
            break;
        }
        if c == '\n' {
            *line += 1;
        }
        result.push(c);
    }
    // read josi
    let josi_opt = josi_list::read_josi(cur);
    let label = result.iter().collect();
    if ex_str {
        return Token::new(TokenKind::StringEx, label, josi_opt, line_begin);
    } else {
        return Token::new(TokenKind::String, label, josi_opt, line_begin);
    }
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
        let t = tokenize("(3.0)");
        assert_eq!(tokens_string(&t), "[ParenL:(][Number:3.0][ParenR:)]");
        let t = tokenize("A=3*5");
        assert_eq!(tokens_string(&t), "[Word:A][Flag:=][Int:3][Flag:*][Int:5]");
    }
    #[test]
    fn test_tokenize_josi() {
        let t = tokenize("AからBまで");
        assert_eq!(tokens_string(&t), "[Word:A/から][Word:B/まで]");
        let t = tokenize("犬をネコへ");
        assert_eq!(tokens_string(&t), "[Word:犬/を][Word:ネコ/へ]");
    }
    #[test]
    fn test_tokenize_str() {
        let t = tokenize("35から「abc」まで置換");
        assert_eq!(tokens_string(&t), "[Int:35/から][StringEx:abc/まで][Word:置換]");
        let t = tokenize("「１２３123」");
        assert_eq!(tokens_string(&t), "[StringEx:１２３123]");
        let t = tokenize("'hoge'");
        assert_eq!(tokens_string(&t), "[String:hoge]");
        let t = tokenize("『boo』");
        assert_eq!(tokens_string(&t), "[String:boo]");
    }

    #[test]
    fn test_delete_okurigana() {
        assert_eq!(delete_okurigana_str("切取り"), String::from("切取"));
        assert_eq!(delete_okurigana_str("置き換える"), String::from("置換"));
        assert_eq!(delete_okurigana_str("なでしこ"), String::from("なでしこ"));
        assert_eq!(delete_okurigana_str("お兄ちゃん"), String::from("お兄"));
        assert_eq!(delete_okurigana_str("F価格"), String::from("F価格"));
        assert_eq!(delete_okurigana_str("VS食べる"), String::from("VS食"));
        assert_eq!(delete_okurigana_str("INTする"), String::from("INT"));
    }

    #[test]
    fn test_reserved_word() {
        let t = tokenize("35回");
        assert_eq!(tokens_string(&t), "[Int:35][Repeat]");
    }
}

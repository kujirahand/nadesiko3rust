//! 軸解析器

use crate::prepare;
use crate::strcur::StrCur;
use crate::kanautils;
use crate::josi_list;
use crate::reserved_words;
use crate::token::*;

/// 文字列をトークンに区切る
pub fn tokenize(src: &str) -> Vec<Token> {
    // 普通にトークンに区切る
    let tok: Vec<Token> = tokenize_src(src, 1);
    
    // 助詞の「は」を＝に展開する
    let mut last_is_eq = false;
    let mut result: Vec<Token> = vec![];
    for t in tok.into_iter() {
        if last_is_eq {
            last_is_eq = false;
            if t.kind == TokenKind::Comma { continue; } // 「Aは,1」のような場合にカンマを飛ばす
        }
        match &t.josi {
            Some(j) => {
                let line = t.line;
                if j.eq("は") {
                    let mut t2 = t.clone();
                    t2.josi = None;
                    result.push(t2);
                    result.push(Token::new_char(TokenKind::Eq, '=', line));
                    last_is_eq = true;
                    continue;
                }    
            }
            None => {},
        }
        result.push(t);
    }
    result
}

pub fn tokenize_src(src: &str, line_begin: u32) -> Vec<Token> {
    let src = prepare::convert(src);
    let mut cur = StrCur::from(&src);
    let mut result: Vec<Token> = vec![];
    let mut line = line_begin;
    while cur.can_read() {
        if cur.skip_space() { continue; }
        let ch = cur.peek();
        match ch {
            '\n' => { result.push(read_lf(&mut cur, &mut line)); continue; },
            ';' => {
                if cur.eq_str(";;;") { // 「。。。」と「ここまで」は同じ意味
                    flag_push_n(TokenKind::BlockEnd, ';', &mut result, &mut cur, 3, line);
                    continue;
                }
                flag_push(TokenKind::Eol, &mut result, &mut cur, line); continue;
            },
            '💧' => { flag_push(TokenKind::BlockEnd, &mut result, &mut cur, line); continue; }
            ',' => { flag_push(TokenKind::Comma, &mut result, &mut cur, line); continue; },
            '/' => { result.push(read_slash(&mut cur, &mut line)); continue; },
            '※' => { result.push(read_linecomment(&mut cur, &mut line)); continue; },
            '#' => { result.push(read_linecomment(&mut cur, &mut line)); continue; },
            // 文字列記号
            '「' => { read_string(&mut result, &mut cur, &mut line, '」', true); continue; }
            '『' => { read_string(&mut result, &mut cur, &mut line, '』', false); continue; }
            '"' => { read_string(&mut result, &mut cur, &mut line, '"', true); continue; }
            '\'' => { read_string(&mut result, &mut cur, &mut line, '\'', false); continue; }
            //各種カッコ
            '(' => { flag_push(TokenKind::ParenL, &mut result, &mut cur, line); continue; },
            ')' => { flag_push_josi(TokenKind::ParenR, &mut result, &mut cur, line); continue; },
            '[' => { flag_push(TokenKind::BracketL, &mut result, &mut cur, line); continue; },
            ']' => { flag_push_josi(TokenKind::BracketR, &mut result, &mut cur, line); continue; },
            '{' => { flag_push(TokenKind::CurBracketL, &mut result, &mut cur, line); continue; },
            '}' => { flag_push_josi(TokenKind::CurBracketR, &mut result, &mut cur, line); continue; },
            // 演算子
            '+' => { flag_push(TokenKind::Plus, &mut result, &mut cur, line); continue; },
            '-' => { flag_push(TokenKind::Minus, &mut result, &mut cur, line); continue; },
            '*' => { flag_push(TokenKind::Mul, &mut result, &mut cur, line); continue; },
            '×' => { flag_push_n(TokenKind::Mul, '*', &mut result, &mut cur, 1, line); continue; },
            '÷' => { flag_push_n(TokenKind::Div, '/', &mut result, &mut cur, 1, line); continue; },
            '%' => { flag_push(TokenKind::Mod, &mut result, &mut cur, line); continue; },
            '^' => { flag_push(TokenKind::Pow, &mut result, &mut cur, line); continue; },
            '\\' => { flag_push(TokenKind::Flag, &mut result, &mut cur, line); continue; },
            '`' => { flag_push(TokenKind::Flag, &mut result, &mut cur, line); continue; },
            '~' => { flag_push(TokenKind::Flag, &mut result, &mut cur, line); continue; },
            '≧' => { flag_push(TokenKind::GtEq, &mut result, &mut cur, line); continue; },
            '≦' => { flag_push(TokenKind::LtEq, &mut result, &mut cur, line); continue; },
            '≠' => { flag_push(TokenKind::NotEq, &mut result, &mut cur, line); continue; },
            '真' => { flag_push_josi(TokenKind::True, &mut result, &mut cur, line); continue; },
            '偽' => { flag_push_josi(TokenKind::False, &mut result, &mut cur, line); continue; },
            '=' => {
                if cur.eq_str("==") { flag_push_n(TokenKind::Eq, '=', &mut result, &mut cur, 2, line); }
                else { flag_push_n(TokenKind::Eq, '=', &mut result, &mut cur, 1, line); }
                continue; 
            },
            '&' => { 
                if cur.eq_str("&&") { flag_push_n(TokenKind::And, '&', &mut result, &mut cur, 2, line); }
                else { flag_push_n(TokenKind::PlusStr, '結', &mut result, &mut cur, 1, line); }
                continue; 
            },
            '|' => { 
                if cur.eq_str("||") { flag_push_n(TokenKind::Or, '|', &mut result, &mut cur, 2, line); }
                else { flag_push_n(TokenKind::Or, '|', &mut result, &mut cur, 1, line); }
                continue; 
            },
            '!' => {
                if cur.eq_str("!=") { flag_push_n(TokenKind::NotEq, '≠', &mut result, &mut cur, 2, line); }
                else { flag_push(TokenKind::Not, &mut result, &mut cur, line); }
                continue; 
            },
            '>' => {
                if cur.eq_str(">=") { flag_push_n(TokenKind::GtEq, '≧', &mut result, &mut cur, 2, line); }
                else if cur.eq_str("><") { flag_push_n(TokenKind::NotEq, '≠', &mut result, &mut cur, 2, line); cur.next(); }
                else { flag_push(TokenKind::Gt, &mut result, &mut cur, line); }
                continue;
            },
            '<' => {
                if cur.eq_str("<=") { flag_push_n(TokenKind::LtEq, '≦', &mut result, &mut cur, 2, line); }
                else if cur.eq_str("<>") { flag_push_n(TokenKind::NotEq, '≠', &mut result, &mut cur, 2, line); }
                else { flag_push(TokenKind::Lt, &mut result, &mut cur, line); }
                continue;
            },
            '●' => { flag_push(TokenKind::DefFunc, &mut result, &mut cur, line); continue; },
            // '!'..='.' => { flag_push(TokenKind::Flag, &mut result, &mut cur, line); continue; },
            // ':'..='@' => { flag_push(TokenKind::Flag, &mut result, &mut cur, line); continue; },
            // 数値
            '0'..='9' => { result.push(read_number(&mut cur, &mut line)); continue; },
            // word
            'a'..='z' | 'A'..='Z' | '_' => { read_word(&mut result, &mut cur, &mut line); continue; }
            n if n > (0xE0 as char) => { read_word(&mut result, &mut cur, &mut line); continue; }
            _ => {} // pass
        }
        // pass
        println!("[字句解析エラー]: 未定義の文字『{}』", ch);
        cur.next();
    }
    result
}

// 1文字をトークンとして追加する関数
fn flag_push(kind: TokenKind, result: &mut Vec<Token>, cur: &mut StrCur, line: u32) {
    let tok = Token {
        kind,
        label: String::from(cur.next()),
        josi: None,
        line,
    };
    result.push(tok);   
}
fn flag_push_josi(kind: TokenKind, result: &mut Vec<Token>, cur: &mut StrCur, line: u32) {
    let label = String::from(cur.next());
    let josi_opt = josi_list::read_josi(cur);
    let tok = Token {
        kind,
        label,
        josi: josi_opt,
        line,
    };
    result.push(tok);   
}
// len文字をトークンとして追加する関数
fn flag_push_n(kind: TokenKind, flag_ch: char, result: &mut Vec<Token>, cur: &mut StrCur, len: usize, line: u32) {
    let tok = Token {
        kind,
        label: String::from(flag_ch),
        josi: None,
        line,
    };
    cur.seek(len as i32);
    result.push(tok);   
}

fn read_lf(cur: &mut StrCur, line: &mut u32) -> Token {
    let lf = cur.next();
    let t = Token::new_char(TokenKind::Eol, lf, *line);
    *line += 1;
    return t;
}

fn read_linecomment(cur: &mut StrCur, line: &mut u32) -> Token {
    cur.seek(1); // skip "※"
    let rem = cur.get_token_tostr('\n');
    let tok = Token::new_str(TokenKind::Comment, &rem, *line);
    *line += 1;
    return tok;
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
    return Token::new_char(TokenKind::Div, flag, *line);
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

fn check_special(result: &mut Vec<Token>, cur: &mut StrCur, word: &str, kind: TokenKind, reg_word: &str, line: u32) -> bool {
    if cur.eq_str(word) {
        let len = word.chars().count();
        cur.seek(len as i32);
        let tok = Token::new_str(kind, reg_word, line);
        result.push(tok);
        return true;
    }
    false
}

fn read_word(result: &mut Vec<Token>, cur: &mut StrCur, line: &mut u32) -> bool {
    let mut word: Vec<char> = vec![];
    let mut josi_opt:Option<String> = None;

    // 特別な語句を例外で登録する
    if cur.eq_str("ここ") {        
        if check_special(result, cur, "ここから", TokenKind::BlockBegin, "ここから", *line) { return true; }
        if check_special(result, cur, "ここまで", TokenKind::BlockEnd, "ここまで", *line) { return true; }
    }
    if cur.eq_str("違") {
        if check_special(result, cur, "違えば", TokenKind::Else, "違", *line) { return true; }
        if check_special(result, cur, "違うなら", TokenKind::Else, "違", *line) { return true; }
    }
    if check_special(result, cur, "または", TokenKind::Or, "||", *line) { return true; }
    if check_special(result, cur, "あるいは", TokenKind::Or, "||", *line) { return true; }
    if check_special(result, cur, "かつ", TokenKind::And, "&&", *line) { return true; }
    if check_special(result, cur, "もしも", TokenKind::If, "もし", *line) { return true; }
    if check_special(result, cur, "もし", TokenKind::If, "もし", *line) { return true; }

    // ひらがなスタートなら1文字目は助詞にならない
    if kanautils::is_hiragana(cur.peek()) {
        word.push(cur.next());
    }
    
    while cur.can_read() {
        let c = cur.peek();
        // 助詞か？
        if kanautils::is_hiragana(c) {
            josi_opt = josi_list::read_josi(cur);
            match josi_opt {
                Some(_) => break, // 助詞なら繰り返しを抜ける
                None => {}, // pass
            }
        }
        // wordになり得る文字か？
        if kanautils::is_word_chars(c) {
            word.push(cur.next());
            continue;
        }
        break;
    }
    
    // 末尾が「回」なら分割、N回→N|回
    let has_kai = if word.last() == Some(&'回') {
        word.pop(); // 回を削除
        true
    } else { false };
    // 送りがなをカット
    word = delete_okurigana(word);
    if word.len() > 0 {
        // トークンを追加
        let word_s: String = word.iter().collect();
        let kind = reserved_words::check_kind(&word_s);
        let tok = Token::new(kind, word_s, josi_opt, *line);
        result.push(tok);
    }
    //　回を追加
    if has_kai {
        let kai_tok = Token::new(TokenKind::Kai, String::from("回"), None, *line);
        result.push(kai_tok);
    }
    true
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
        if !kanautils::is_hiragana(*c) {
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

fn read_string(result: &mut Vec<Token>, cur: &mut StrCur, line: &mut u32, end_flag: char, ex_str: bool) {
    cur.next(); // begin_flag
    let mut res: Vec<char> = vec![];
    let line_begin = *line;
    while cur.can_read() {
        let c = cur.next();
        if c == end_flag {
            break;
        }
        if c == '\n' {
            *line += 1;
        }
        res.push(c);
    }
    // read josi
    let josi_opt = josi_list::read_josi(cur);
    let label = res.iter().collect();
    if ex_str {
        extract_string_ex(result, label, josi_opt, line_begin);
    } else {
        let tok = Token::new(TokenKind::String, label, josi_opt, line_begin);
        result.push(tok);
    }
}

fn extract_string_ex(result: &mut Vec<Token>, src: String, josi_opt:Option<String>, line: u32) {
    let mut data = String::new();
    let mut code = String::new();
    let mut is_extract = false;
    for c in src.chars() {
        if is_extract {
            if c == '}' || c == '｝' {
                let list = tokenize_src(&code, line);
                if list.len() > 0 {
                    result.push(Token::new(TokenKind::PlusStr, String::from("結"), None, line));
                    result.push(Token::new(TokenKind::ParenL, String::from("("), None, line));
                    for t in list.into_iter() {
                        result.push(t);
                    }
                    result.push(Token::new(TokenKind::ParenR, String::from(")"), None, line));
                    result.push(Token::new(TokenKind::PlusStr, String::from("結"), None, line));
                    is_extract = false;
                }
                continue;
            }
            code.push(c);
            continue;
        }
        if c == '{' || c == '｛' {
            is_extract = true;
            result.push(Token::new(TokenKind::String, data, None, line));
            data = String::new();
            continue;
        }
        data.push(c);
    }
    result.push(Token::new(TokenKind::String, data, josi_opt.clone(), line));
}



#[cfg(test)]
mod test_tokenizer {
    use super::*;

    fn delete_okurigana_str(word: &str) -> String {
        let word_v:Vec<char> = word.chars().collect();
        let res_v = delete_okurigana(word_v);
        res_v.iter().collect()
    }
    
    #[test]
    fn test_tokenize() {
        let t = tokenize("//abc");
        assert_eq!(tokens_string(&t), "[Comment:abc]");
        let t = tokenize("//abc\n\n/*ABC*/");
        assert_eq!(tokens_string(&t), "[Comment:abc][Eol][Comment:ABC]");
        let t = tokenize("3\n3.14");
        assert_eq!(tokens_string(&t), "[Int:3][Eol][Number:3.14]");
        let t = tokenize("hoge=35");
        assert_eq!(tokens_string(&t), "[Word:hoge][=][Int:35]");
        let t = tokenize("年齢=15");
        assert_eq!(tokens_string(&t), "[Word:年齢][=][Int:15]");
        let t = tokenize("(3.0)");
        assert_eq!(tokens_string(&t), "[(][Number:3.0][)]");
        let t = tokenize("A=3*5");
        assert_eq!(tokens_string(&t), "[Word:A][=][Int:3][*][Int:5]");
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
        assert_eq!(tokens_string(&t), "[Int:35/から][String:abc/まで][Word:置換]");
        let t = tokenize("「１２３123」");
        assert_eq!(tokens_string(&t), "[String:１２３123]");
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
        assert_eq!(tokens_string(&t), "[Int:35][Kai]");
        let t = tokenize("N回");
        assert_eq!(tokens_string(&t), "[Word:N][Kai]");
    }

    #[test]
    fn test_word_check() {
        let t = tokenize("35回『ワン』と表示");
        assert_eq!(tokens_string(&t), "[Int:35][Kai][String:ワン/と][Word:表示]");
        let t = tokenize("N回");
        assert_eq!(tokens_string(&t), "[Word:N][Kai]");
    }

    #[test]
    fn test_extract_string() {
        let t = tokenize("「a={a}」と表示");
        assert_eq!(tokens_string(&t), "[String:a=][&][(][Word:a][)][&][String:/と][Word:表示]");
    }
}

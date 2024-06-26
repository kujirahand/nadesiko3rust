//! 字句解析器

use crate::prepare;
use crate::strcur::StrCur;
use crate::kanautils;
use crate::josi_list;
use crate::reserved_words;
use crate::token::*;
use crate::nvalue::NValue;
use crate::tokencur::TokenCur;

#[derive(Debug, Clone)]
pub struct Tokenizer {
    pub cur: StrCur,
}

impl Tokenizer {
    /// 新しいインスタンスを生成する
    pub fn new(src: &str, start: i32, fileno: i32) -> Tokenizer {
        let src = prepare::convert(src, fileno);
        Tokenizer {
            cur: StrCur::from_source(&src, start, fileno),
        }
    }
    /// 文字列をトークンに区切る
    pub fn tokenize(&mut self) -> Vec<Token> {
        // 普通にトークンに区切る
        let tok: Vec<Token> = self.split();
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
                    if j.eq("は") {
                        let mut t2 = t.clone();
                        t2.josi = None;
                        result.push(t2);
                        result.push(Token::new_char(TokenKind::Eq, '=', t.pos));
                        last_is_eq = true;
                        continue;
                    }    
                }
                None => {},
            }
            result.push(t);
        }
        // 行番号を計算する
        self.calc_lineno(&mut result);
        result
    }

    /// 文字列を単純にトークンに分割する
    fn split(&mut self) -> Vec<Token> {
        let mut cur = self.cur.clone();
        let mut result: Vec<Token> = vec![];
        while cur.can_read() {
            if cur.skip_space() { continue; }
            let ch = cur.peek();
            match ch {
                '\r' => { cur.next(); continue; }, // skip CR
                '\n' => { result.push(read_lf(&mut cur)); continue; }, // record LF
                ';' => {
                    if cur.eq_str(";;;") { // 「。。。」と「ここまで」は同じ意味
                        flag_push_n(TokenKind::BlockEnd, ';', &mut result, &mut cur, 3);
                        continue;
                    }
                    flag_push(TokenKind::Eol, &mut result, &mut cur); continue;
                },
                '💧' => { flag_push(TokenKind::BlockEnd, &mut result, &mut cur); continue; }
                ',' => { flag_push(TokenKind::Comma, &mut result, &mut cur); continue; },
                '/' => { result.push(read_slash(&mut cur)); continue; },
                '※' => { result.push(read_linecomment(&mut cur)); continue; },
                '#' => { result.push(read_linecomment(&mut cur)); continue; },
                // 文字列記号
                '「' => { self.read_string(&mut result, &mut cur, '」', true); continue; }
                '『' => { self.read_string(&mut result, &mut cur, '』', false); continue; }
                '"' => { self.read_string(&mut result, &mut cur, '"', true); continue; }
                '\'' => { self.read_string(&mut result, &mut cur, '\'', false); continue; }
                //各種カッコ
                '(' => { flag_push(TokenKind::ParenL, &mut result, &mut cur); continue; },
                ')' => { flag_push_josi(TokenKind::ParenR, &mut result, &mut cur); continue; },
                '[' => { flag_push(TokenKind::BracketL, &mut result, &mut cur); continue; },
                ']' => { flag_push_josi(TokenKind::BracketR, &mut result, &mut cur); continue; },
                '{' => { flag_push(TokenKind::CurBracketL, &mut result, &mut cur); continue; },
                '}' => { flag_push_josi(TokenKind::CurBracketR, &mut result, &mut cur); continue; },
                // 演算子
                '+' => { flag_push(TokenKind::Plus, &mut result, &mut cur); continue; },
                '-' => { flag_push(TokenKind::Minus, &mut result, &mut cur); continue; },
                '*' => { flag_push(TokenKind::Mul, &mut result, &mut cur); continue; },
                '×' => { flag_push_n(TokenKind::Mul, '*', &mut result, &mut cur, 1); continue; },
                '÷' => { flag_push_n(TokenKind::Div, '/', &mut result, &mut cur, 1); continue; },
                '%' => { flag_push(TokenKind::Mod, &mut result, &mut cur); continue; },
                '^' => { flag_push(TokenKind::Pow, &mut result, &mut cur); continue; },
                '\\' => { flag_push(TokenKind::Flag, &mut result, &mut cur); continue; },
                '`' => { flag_push(TokenKind::Flag, &mut result, &mut cur); continue; },
                '~' => { flag_push(TokenKind::Flag, &mut result, &mut cur); continue; },
                '≧' => { flag_push(TokenKind::GtEq, &mut result, &mut cur); continue; },
                '≦' => { flag_push(TokenKind::LtEq, &mut result, &mut cur); continue; },
                '≠' => { flag_push(TokenKind::NotEq, &mut result, &mut cur); continue; },
                '真' => { flag_push_josi(TokenKind::True, &mut result, &mut cur); continue; },
                '偽' => { flag_push_josi(TokenKind::False, &mut result, &mut cur); continue; },
                '=' => {
                    if cur.eq_str("==") { flag_push_n(TokenKind::Eq, '=', &mut result, &mut cur, 2); }
                    else { flag_push_n(TokenKind::Eq, '=', &mut result, &mut cur, 1); }
                    continue; 
                },
                '&' => { 
                    if cur.eq_str("&&") { flag_push_n(TokenKind::And, '&', &mut result, &mut cur, 2); }
                    else { flag_push_n(TokenKind::PlusStr, '結', &mut result, &mut cur, 1); }
                    continue; 
                },
                '|' => { 
                    if cur.eq_str("||") { flag_push_n(TokenKind::Or, '|', &mut result, &mut cur, 2); }
                    else { flag_push_n(TokenKind::Or, '|', &mut result, &mut cur, 1); }
                    continue; 
                },
                '!' => {
                    if cur.eq_str("!=") { flag_push_n(TokenKind::NotEq, '≠', &mut result, &mut cur, 2); }
                    else { flag_push(TokenKind::Not, &mut result, &mut cur); }
                    continue; 
                },
                '>' => {
                    if cur.eq_str(">=") { flag_push_n(TokenKind::GtEq, '≧', &mut result, &mut cur, 2); }
                    else if cur.eq_str("><") { flag_push_n(TokenKind::NotEq, '≠', &mut result, &mut cur, 2); cur.next(); }
                    else { flag_push(TokenKind::Gt, &mut result, &mut cur); }
                    continue;
                },
                '<' => {
                    if cur.eq_str("<=") { flag_push_n(TokenKind::LtEq, '≦', &mut result, &mut cur, 2); }
                    else if cur.eq_str("<>") { flag_push_n(TokenKind::NotEq, '≠', &mut result, &mut cur, 2); }
                    else { flag_push(TokenKind::Lt, &mut result, &mut cur); }
                    continue;
                },
                '●' => { flag_push(TokenKind::DefFunc, &mut result, &mut cur); continue; },
                // '!'..='.' => { flag_push(TokenKind::Flag, &mut result, &mut cur); continue; },
                // ':'..='@' => { flag_push(TokenKind::Flag, &mut result, &mut cur); continue; },
                // 数値
                '0'..='9' => { result.push(read_number(&mut cur)); continue; },
                // word
                'a'..='z' | 'A'..='Z' | '_' => { read_word(&mut result, &mut cur); continue; }
                n if n > (0xE0 as char) => { read_word(&mut result, &mut cur); continue; }
                _ => {} // pass
            }
            // pass
            let lineno = cur.get_lineno(cur.get_index_i());
            println!("[字句解析エラー]({}): 未定義の文字『{}』", lineno, ch);
            cur.next();
        }
        self.cur = cur;
        result
    }

    fn read_string(&self, result: &mut Vec<Token>, cur: &mut StrCur, end_flag: char, ex_str: bool) {
        let start = cur.get_index_i();
        cur.next(); // begin_flag
        let mut res: Vec<char> = vec![];
        while cur.can_read() {
            let c = cur.next();
            if c == end_flag {
                break;
            }
            res.push(c);
        }
        // read josi
        let josi_opt = josi_list::read_josi(cur);
        let label = res.iter().collect();
        if ex_str {
            self.extract_string_ex(result, label, josi_opt, start, cur.fileno);
        } else {
            let end = cur.get_index_i();
            let tok = Token::new(TokenKind::String, NValue::String(label), josi_opt, TokenPos::new(start, end, cur.fileno));
            result.push(tok);
        }
    }

    fn extract_string_ex(&self, result: &mut Vec<Token>, src: String, josi_opt:Option<String>, start: i32, fileno: i32) {
        let mut data = String::new();
        let mut code = String::new();
        let mut is_extract = false;
        let mut last_index = 0;
        for (index, c) in src.chars().enumerate() {
            if is_extract {
                if c == '}' || c == '｝' {
                    last_index = index + 1;
                    let mut toknizer = Tokenizer::new(&code, start, fileno);
                    toknizer.cur.top_index = (start + last_index as i32) as i32;
                    let list = toknizer.tokenize();
                    if list.len() > 0 {
                        let pos = list[0].pos;
                        let end_pos = list[list.len() - 1].pos.clone();
                        result.push(Token::new(TokenKind::PlusStr, NValue::from_char('結'), None, pos));
                        result.push(Token::new(TokenKind::ParenL, NValue::from_char('('), None, pos));
                        for t in list.into_iter() {
                            result.push(t);
                        }
                        result.push(Token::new(TokenKind::ParenR, NValue::from_char(')'), None, end_pos));
                        result.push(Token::new(TokenKind::PlusStr, NValue::from_char('結'), None, end_pos));
                        is_extract = false;
                    }
                    continue;
                }
                code.push(c);
                continue;
            }
            if c == '{' || c == '｛' {
                is_extract = true;
                let end = index as i32;
                let begin_pos = TokenPos::new(start + last_index as i32, start + end, fileno);
                result.push(Token::new(
                    TokenKind::String,
                    NValue::String(data), None,
                    begin_pos));
                data = String::new();
                continue;
            }
            data.push(c);
        }
        let src_len = src.chars().count() as i64;
        result.push(
            Token::new(
                TokenKind::String, NValue::String(data), josi_opt.clone(), TokenPos::new(start + last_index as i32, start + src_len as i32, fileno)
            ));
    }

    /// ソースから行番号を計算する
    pub fn calc_lineno(&mut self, tokens: &mut Vec<Token>) {
        // 行番号を計算する
        let mut lineno = 1;
        let mut col = 1;
        // 最初に行番号とインデックスの対応表を作る
        let mut rows_vec: Vec<i32> = vec![0; self.cur.src.len()];
        let mut cols_vec: Vec<i32> = vec![0; self.cur.src.len()];
        for (i, c) in self.cur.src.iter().enumerate() {
            rows_vec[i] = lineno;
            cols_vec[i] = col;
            if *c == '\n' {
                lineno += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        // 対応表を元にしてトークンに行番号を入れる
        for tok in tokens.iter_mut() {
            let pos = tok.pos.start as usize;
            if pos < rows_vec.len() {
                tok.pos.row = rows_vec[pos];
                tok.pos.col = cols_vec[pos]
            }
        }
    }
}


// 1文字をトークンとして追加する関数
fn flag_push(kind: TokenKind, result: &mut Vec<Token>, cur: &mut StrCur) {
    let start = cur.get_index_i();
    let tok = Token {
        kind,
        value: NValue::from_char(cur.next()),
        josi: None,
        pos: TokenPos::new(start, start + 1, cur.fileno),
    };
    result.push(tok);
}
fn flag_push_josi(kind: TokenKind, result: &mut Vec<Token>, cur: &mut StrCur) {
    let start = cur.get_index_i();
    let label = cur.next();
    let josi_opt = josi_list::read_josi(cur);
    let tok = Token {
        kind,
        value: NValue::from_char(label),
        josi: josi_opt,
        pos: TokenPos::new(start, start + 1, cur.fileno),
    };
    result.push(tok);   
}
// len文字をトークンとして追加する関数
fn flag_push_n(kind: TokenKind, flag_ch: char, result: &mut Vec<Token>, cur: &mut StrCur, len: usize) {
    let start = cur.get_index_i();
    cur.seek(len as i64);
    let end = cur.get_index_i();
    let tok = Token {
        kind,
        value: NValue::from_char(flag_ch),
        josi: None,
        pos: TokenPos::new(start, end, cur.fileno),
    };
    result.push(tok);
}

fn read_lf(cur: &mut StrCur) -> Token {
    let start = cur.get_index_i();
    cur.next(); // skip LF
    // 連続する改行をスキップ
    while cur.can_read() {
        let c = cur.peek();
        if c == '\n' || c == '\r' {
            cur.next();
            continue;
        }
        break;
    }
    Token::new_char(TokenKind::Eol, '\n', TokenPos::new(start, start + 1, cur.fileno))
}

fn read_linecomment(cur: &mut StrCur) -> Token {
    let start = cur.get_index_i();
    cur.seek(1); // skip "※"
    let rem = cur.get_token_tostr('\n');
    let end = cur.get_index_i();
    let tok = Token::new(TokenKind::Comment, NValue::String(rem), None, TokenPos::new(start, end, cur.fileno));
    return tok;
}

fn read_slash(cur: &mut StrCur) -> Token {
    // line comment
    if cur.eq_str("//") {
        let start = cur.get_index_i();
        cur.seek(2); // skip "//"
        let rem = cur.get_token_tostr('\n');
        let end = cur.get_index_i();
        let tok = Token::new_str(TokenKind::Comment, &rem, TokenPos::new(start, end, cur.fileno));
        return tok;
    }
    // range comment
    if cur.eq_str("/*") {
        let start = cur.get_index_i();
        cur.seek(2); // skio "/*"
        let rem = cur.get_token_str("*/");
        let end = cur.get_index_i();
        let rem_s: String = rem.iter().collect();
        let tok = Token::new_str(TokenKind::Comment, &rem_s, TokenPos::new(start, end, cur.fileno));
        return tok;
    }
    // flag
    let start = cur.get_index_i();
    let flag = cur.next();
    let end = cur.get_index_i();
    return Token::new_char(TokenKind::Div, flag, TokenPos::new(start, end, cur.fileno));
}

fn read_number(cur: &mut StrCur) -> Token {
    let start = cur.get_index_i();
    let mut vc: Vec<char> = vec![];
    while cur.peek_in_range('0', '9') {
        vc.push(cur.next());
    }
    // float value
    if cur.peek() == '.' {
        vc.push(cur.next());
        while cur.peek_in_range('0', '9') {
            vc.push(cur.next());
        }
        let num_s: String = vc.iter().collect();
        let josi_opt = josi_list::read_josi(cur);
        let end = cur.get_index_i();
        let nv = NValue::from_float(NValue::from_string(num_s).to_float_def(0.0));
        return Token::new(TokenKind::Number, nv, josi_opt, TokenPos::new(start, end, cur.fileno));
    }
    // int value
    let num_s: String = vc.iter().collect();
    let josi_opt = josi_list::read_josi(cur);
    let end = cur.get_index_i();
    let nv = NValue::from_int(NValue::from_string(num_s).to_int_def(0));
    return Token::new(TokenKind::Int, nv, josi_opt, TokenPos::new(start, end, cur.fileno));
}

fn check_special(result: &mut Vec<Token>, cur: &mut StrCur, word: &str, kind: TokenKind, reg_word: &str) -> bool {
    let start = cur.get_index_i();
    if cur.eq_str(word) {
        let len = word.chars().count();
        cur.seek(len as i64);
        let end = cur.get_index_i();
        let tok = Token::new_str(kind, reg_word, TokenPos::new(start, end, cur.fileno));
        result.push(tok);
        return true;
    }
    false
}

fn read_word(result: &mut Vec<Token>, cur: &mut StrCur) -> bool {
    let mut word: Vec<char> = vec![];
    let mut josi_opt:Option<String> = None;
    let start = cur.get_index_i();

    // 特別な語句を例外で登録する
    if cur.eq_str("ここ") {        
        if check_special(result, cur, "ここから", TokenKind::BlockBegin, "ここから") { return true; }
        if check_special(result, cur, "ここまで", TokenKind::BlockEnd, "ここまで") { return true; }
    }
    if cur.eq_str("違") {
        if check_special(result, cur, "違えば", TokenKind::Else, "違") { return true; }
        if check_special(result, cur, "違うなら", TokenKind::Else, "違") { return true; }
    }
    if check_special(result, cur, "または", TokenKind::Or, "||") { return true; }
    if check_special(result, cur, "あるいは", TokenKind::Or, "||") { return true; }
    if check_special(result, cur, "かつ", TokenKind::And, "&&") { return true; }
    if check_special(result, cur, "もしも", TokenKind::If, "もし") { return true; }
    if check_special(result, cur, "もし", TokenKind::If, "もし") { return true; }

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
        let end = cur.get_index_i();
        let tok = Token::new(kind, NValue::from_string(word_s), josi_opt, TokenPos::new(start, end, cur.fileno));
        result.push(tok);
    }
    //　回を追加
    if has_kai {
        let end = cur.get_index_i();
        let kai_tok = Token::new(TokenKind::Kai, NValue::from_str("回"), None, TokenPos::new(start, end, cur.fileno));
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

pub fn tokenize(src: &str, start: i32, fileno: i32) -> Vec<Token> {
    let mut t = Tokenizer::new(src, start, fileno);
    t.tokenize()
}

pub fn tokenize_test(src: &str) -> Vec<Token> {
    tokenize(src, 0, 0)
}

// 『！「＊＊＊」を取り込む』を先読みする
pub fn read_include_files(tokens: Vec<Token>) -> (Vec<Token>, Vec<String>) {
    let mut result = vec![];
    let mut flag_line_top = true;
    let mut cur = TokenCur::new(tokens);
    while cur.can_read() {
        // 行頭の「!」かどうかを判定
        let t = cur.peek();
        if flag_line_top && t.value.eq_char('!') {
            let top_index = cur.index;
            let flag_t = cur.next(); // skip !
            // ! "str" word
            if cur.eq_kinds(&[TokenKind::String, TokenKind::Word]) {
                let file_t = cur.next();
                let do_t = cur.next();
                // 取り込む？
                if do_t.value.eq_str("取込") {
                    let file_path = file_t.value.to_string();
                    result.push(file_path.clone());
                    // トークンをコメントに置換
                    let include_comment = format!("!「{}」を取込", file_path);
                    cur.replace_token(top_index + 0, Token::new_comment(&include_comment, flag_t.pos));
                    cur.replace_token(top_index + 1, Token::new_comment("", flag_t.pos));
                    cur.replace_token(top_index + 2, Token::new_comment("", flag_t.pos));
                    continue;
                }
            }
        }
        // 行頭かどうかを確認
        if cur.eq_kind(TokenKind::Eol) {
            flag_line_top = true;
        } else {
            flag_line_top = false;
        }
        cur.next();
    }
    (cur.tokens, result)
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
        let t = tokenize_test("//abc");
        assert_eq!(tokens_string(&t), "[Comment:abc]");
        assert_eq!(tokens_string_pos(&t), "[Comment:abc](0,5)");
        let t = tokenize_test("//abc\n\n/*ABC*/");
        assert_eq!(tokens_string(&t), "[Comment:abc][Eol][Comment:ABC]");
        let t = tokenize_test("3\n3.14");
        assert_eq!(tokens_string(&t), "[Int:3][Eol][Number:3.14]");
        assert_eq!(tokens_string_pos(&t), "[Int:3](0,1)[Eol](1,2)[Number:3.14](2,6)");
        let t = tokenize_test("hoge=35");
        assert_eq!(tokens_string(&t), "[Word:hoge][=][Int:35]");
        assert_eq!(tokens_string_pos(&t), "[Word:hoge](0,4)[=](4,5)[Int:35](5,7)");
        let t = tokenize_test("年齢=15");
        assert_eq!(tokens_string(&t), "[Word:年齢][=][Int:15]");
        assert_eq!(tokens_string_pos(&t), "[Word:年齢](0,2)[=](2,3)[Int:15](3,5)");
        let t = tokenize_test("(3.2)");
        assert_eq!(tokens_string(&t), "[(][Number:3.2][)]");
        let t = tokenize_test("A=3*5");
        assert_eq!(tokens_string(&t), "[Word:A][=][Int:3][*][Int:5]");
        assert_eq!(tokens_string_pos(&t), "[Word:A](0,1)[=](1,2)[Int:3](2,3)[*](3,4)[Int:5](4,5)");
        let t = tokenize_test("A = 3 * 5");
        assert_eq!(tokens_string(&t), "[Word:A][=][Int:3][*][Int:5]");
        assert_eq!(tokens_string_pos(&t), "[Word:A](0,1)[=](2,3)[Int:3](4,5)[*](6,7)[Int:5](8,9)");
    }
    #[test]
    fn test_tokenize_josi() {
        let t = tokenize_test("AからBまで");
        assert_eq!(tokens_string(&t), "[Word:A/から][Word:B/まで]");
        let t = tokenize_test("犬をネコへ");
        assert_eq!(tokens_string(&t), "[Word:犬/を][Word:ネコ/へ]");
    }
    #[test]
    fn test_tokenize_str() {
        let t = tokenize_test("35から「abc」まで置換");
        assert_eq!(tokens_string(&t), "[Int:35/から][String:abc/まで][Word:置換]");
        let t = tokenize_test("「１２３123」");
        assert_eq!(tokens_string(&t), "[String:１２３123]");
        let t = tokenize_test("'hoge'");
        assert_eq!(tokens_string(&t), "[String:hoge]");
        let t = tokenize_test("『boo』");
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
        let t = tokenize_test("35回");
        assert_eq!(tokens_string(&t), "[Int:35][Kai]");
        let t = tokenize_test("N回");
        assert_eq!(tokens_string(&t), "[Word:N][Kai]");
    }

    #[test]
    fn test_word_check() {
        let t = tokenize_test("35回『ワン』と表示");
        assert_eq!(tokens_string(&t), "[Int:35][Kai][String:ワン/と][Word:表示]");
        let t = tokenize_test("N回");
        assert_eq!(tokens_string(&t), "[Word:N][Kai]");
    }

    #[test]
    fn test_extract_string() {
        let t = tokenize_test("「a={a}」と表示");
        assert_eq!(tokens_string(&t), "[String:a=][&][(][Word:a][)][&][String:/と][Word:表示]");
    }

    #[test]
    fn test_calc_lineno() {
        let t = tokenize_test("A=0\nB=1\nC=1");
        assert_eq!(tokens_string(&t), "[Word:A][=][Int:0][Eol][Word:B][=][Int:1][Eol][Word:C][=][Int:1]");
        assert_eq!(tokens_string_lineno(&t), "[Word:A](1)[=](1)[Int:0](1)[Eol](1)[Word:B](2)[=](2)[Int:1](2)[Eol](2)[Word:C](3)[=](3)[Int:1](3)");
    }

}

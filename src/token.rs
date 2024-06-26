//! トークンを定義したもの

use crate::nvalue::NValue;

/// トークンの一覧
#[derive(Debug,Clone,PartialEq,Copy)]
pub enum TokenKind {
    None,
    Comment,
    Comma,
    Eol,
    Int,
    Number,
    String,
    Word,
    Flag,
    Eq,
    NotEq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    Not,
    ParenL,
    ParenR,
    BracketL,
    BracketR,
    CurBracketL,
    CurBracketR,
    BlockBegin,
    BlockEnd,
    If,
    Else,
    Kai,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Pow,
    PlusStr,
    And,
    Or,
    True,
    False,
    Break,
    Continue,
    For,
    DefFunc,
    Return,
    DefVar,
    Dainyu,
}

/// トークンのソースコード情報を表現する構造体
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct TokenPos {
    pub start: i32,
    pub end: i32,
    pub fileno: i32,
    pub row: i32, // 字句解析した後で設定する
    pub col: i32, // 字句解析した後で設定する
}

impl TokenPos {
    pub fn new(start: i32, end: i32, fileno: i32) -> Self {
        Self { start, end, fileno, row: 0, col: 0 }
    }
    pub fn to_string_se(&self) -> String {
        format!("({},{})", self.start, self.end)
    }
}

/// トークンを表現する構造体
#[derive(Debug,Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: NValue,
    pub josi: Option<String>,
    pub pos: TokenPos,
}

impl Token {
    /// new token
    pub fn new(kind: TokenKind, value:NValue, josi: Option<String>, pos: TokenPos) -> Self {
        Self { kind, value, josi, pos }
    }
    /// new empty token
    pub fn new_empty() -> Self {
        Self::new(TokenKind::None, NValue::Empty, None, TokenPos::new(0, 0, 0))
    }
    /// new comment token
    pub fn new_comment(comment: &str, pos: TokenPos) -> Self {
        Self::new(TokenKind::Comment, NValue::from_str(comment), None, pos)
    }
    /// new token form char
    pub fn new_char(kind: TokenKind, label: char, pos: TokenPos) -> Self {
        Self {
            kind,
            value: NValue::from_char(label),
            josi: None,
            pos,
        }
    }
    /// new token from string
    pub fn new_str(kind: TokenKind, label: &str, pos: TokenPos) -> Self {
        Self {
            kind,
            value: NValue::from_str(label),
            josi: None,
            pos,
        }
    }
    pub fn as_char(&self) -> char {
        match &self.value {
            NValue::String(c) => if c.len() > 0 { c.chars().nth(0).unwrap_or('\0') } else { '\0' },
            _ => '\0',
        }
    }
    pub fn as_label(&self) -> String {
        self.value.to_string()
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // 助詞の有無に生じて出力方式を変更する
        let get_value = |t: &Token| -> String {
            match &t.josi {
                Some(j) => { format!("{}/{}", t.value.to_string(), j) },
                None    => { format!("{}", t.value.to_string()) },
            }
        };
        let t = &self;
        let s: String = match self.kind {
            TokenKind::None => format!("None"),
            TokenKind::Comment => format!("Comment:{}", get_value(t)),
            TokenKind::Comma => format!(","),
            TokenKind::Eol => format!("Eol"),
            TokenKind::Int => format!("Int:{}", get_value(t)),
            TokenKind::Number => format!("Number:{}", get_value(t)),
            TokenKind::String => format!("String:{}", get_value(t)),
            TokenKind::Word => format!("Word:{}", get_value(t)),
            TokenKind::Flag => format!("Flag:{}", get_value(t)),
            TokenKind::ParenL => String::from("("),
            TokenKind::ParenR => String::from(")"),
            TokenKind::Eq => format!("="),
            TokenKind::NotEq => format!("≠"),
            TokenKind::Plus => format!("+"),
            TokenKind::Minus => format!("-"),
            TokenKind::Mul => format!("*"),
            TokenKind::Div => format!("/"),
            TokenKind::Mod => format!("%"),
            TokenKind::Pow => format!("^"),
            TokenKind::Gt => format!(">"),
            TokenKind::GtEq => format!("≧"),
            TokenKind::Lt => format!("<"),
            TokenKind::LtEq => format!("≦"),
            TokenKind::Not => format!("!"),
            TokenKind::If => format!("もし"),
            TokenKind::Else => format!("違えば"),
            TokenKind::Kai => format!("Kai"),
            TokenKind::BlockBegin => format!("ここから"),
            TokenKind::BlockEnd => format!("ここまで"),
            TokenKind::BracketL => String::from("["),
            TokenKind::BracketR => String::from("]"),
            TokenKind::CurBracketL => String::from("{"),
            TokenKind::CurBracketR => String::from("}"),
            TokenKind::True => String::from("真"),
            TokenKind::False => String::from("偽"),
            TokenKind::And => String::from("&&"),
            TokenKind::Or => String::from("||"),
            TokenKind::PlusStr => String::from("&"),
            TokenKind::Break => String::from("抜"),
            TokenKind::Continue => String::from("続"),
            TokenKind::For => String::from("繰返"),
            TokenKind::DefFunc => String::from("●関数"),
            TokenKind::Return => String::from("戻"),
            TokenKind::DefVar => String::from("変数"),
            TokenKind::Dainyu => String::from("代入"),
            // _ => format!("{:?}", self),
        };
        write!(f, "{}", s)
    }
}

#[allow(dead_code)]
pub fn tokens_string(vt: &[Token]) -> String {
    let mut res = String::new();
    for tok in vt.iter() {
        let s = format!("[{}]", tok);
        res.push_str(&s);
    }
    format!("{}", res)
}

#[allow(dead_code)]
pub fn tokens_string_pos(vt: &[Token]) -> String {
    let mut res = String::new();
    for tok in vt.iter() {
        let s = format!("[{}]{}", tok, tok.pos.to_string_se());
        res.push_str(&s);
    }
    format!("{}", res)
}

#[allow(dead_code)]
pub fn tokens_string_lineno(vt: &[Token]) -> String {
    let mut res = String::new();
    for tok in vt.iter() {
        let s = format!("[{}]({})", tok, tok.pos.row);
        res.push_str(&s);
    }
    format!("{}", res)
}

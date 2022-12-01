//! トークンを定義したもの

/// トークンの一覧
#[derive(Debug,Clone,PartialEq,Copy)]
pub enum TokenType {
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

/// トークンを表現する構造体
#[derive(Debug,Clone)]
pub struct Token {
    pub ttype: TokenType,
    pub label: String,
    pub josi: Option<String>,
    pub line: u32,
}

impl Token {
    pub fn new(kind: TokenType, label: String, josi: Option<String>, line: u32) -> Self {
        Self { ttype: kind, label, josi, line }
    }
    pub fn new_char(kind: TokenType, label: char, line: u32) -> Self {
        Self {
            ttype: kind,
            label: String::from(label),
            josi: None,
            line,
        }
    }
    pub fn new_str(kind: TokenType, label: &str, line: u32) -> Self {
        Self {
            ttype: kind,
            label: String::from(label),
            josi: None,
            line,
        }
    }
    pub fn as_char(&self) -> char {
        if self.label.len() > 0 {
            return self.label.chars().nth(0).unwrap_or('\0');
        }
        '\0'
    }
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
        let s: String = match self.ttype {
            TokenType::None => format!("None"),
            TokenType::Comment => format!("Comment:{}", get_value(t)),
            TokenType::Comma => format!(","),
            TokenType::Eol => format!("Eol"),
            TokenType::Int => format!("Int:{}", get_value(t)),
            TokenType::Number => format!("Number:{}", get_value(t)),
            TokenType::String => format!("String:{}", get_value(t)),
            TokenType::Word => format!("Word:{}", get_value(t)),
            TokenType::Flag => format!("Flag:{}", get_value(t)),
            TokenType::ParenL => String::from("("),
            TokenType::ParenR => String::from(")"),
            TokenType::Eq => format!("="),
            TokenType::NotEq => format!("≠"),
            TokenType::Plus => format!("+"),
            TokenType::Minus => format!("-"),
            TokenType::Mul => format!("*"),
            TokenType::Div => format!("/"),
            TokenType::Mod => format!("%"),
            TokenType::Pow => format!("^"),
            TokenType::Gt => format!(">"),
            TokenType::GtEq => format!("≧"),
            TokenType::Lt => format!("<"),
            TokenType::LtEq => format!("≦"),
            TokenType::Not => format!("!"),
            TokenType::If => format!("もし"),
            TokenType::Else => format!("違えば"),
            TokenType::Kai => format!("Kai"),
            TokenType::BlockBegin => format!("ここから"),
            TokenType::BlockEnd => format!("ここまで"),
            TokenType::BracketL => String::from("["),
            TokenType::BracketR => String::from("]"),
            TokenType::CurBracketL => String::from("{"),
            TokenType::CurBracketR => String::from("}"),
            TokenType::True => String::from("真"),
            TokenType::False => String::from("偽"),
            TokenType::And => String::from("&&"),
            TokenType::Or => String::from("||"),
            TokenType::PlusStr => String::from("&"),
            TokenType::Break => String::from("抜"),
            TokenType::Continue => String::from("続"),
            TokenType::For => String::from("繰返"),
            TokenType::DefFunc => String::from("●関数"),
            TokenType::Return => String::from("戻"),
            TokenType::DefVar => String::from("変数"),
            TokenType::Dainyu => String::from("代入"),
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

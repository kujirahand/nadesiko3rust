// トークン

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
    Eq,
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
    Repeat,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Pow,
    And,
}

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
        let s: String = match self.kind {
            TokenKind::None => format!("None"),
            TokenKind::Comment => format!("Comment:{}", get_value(t)),
            TokenKind::Eol => format!("Eol"),
            TokenKind::Int => format!("Int:{}", get_value(t)),
            TokenKind::Number => format!("Number:{}", get_value(t)),
            TokenKind::String => format!("String:{}", get_value(t)),
            TokenKind::StringEx => format!("StringEx:{}", get_value(t)),
            TokenKind::Word => format!("Word:{}", get_value(t)),
            TokenKind::Flag => format!("Flag:{}", get_value(t)),
            TokenKind::ParenL => format!("ParenL:{}", get_value(t)),
            TokenKind::ParenR => format!("ParenR:{}", get_value(t)),
            TokenKind::Eq => format!("Eq"),
            TokenKind::Plus => format!("+"),
            TokenKind::Minus => format!("-"),
            TokenKind::Mul => format!("*"),
            TokenKind::Div => format!("/"),
            TokenKind::Mod => format!("%"),
            TokenKind::Pow => format!("^"),
            TokenKind::And => format!("&"),
            TokenKind::If => format!("もし"),
            TokenKind::Else => format!("違えば"),
            TokenKind::Repeat => format!("Repeat"),
            TokenKind::BlockBegin => format!("ここから"),
            TokenKind::BlockEnd => format!("ここまで"),
            _ => format!("{:?}", self),
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

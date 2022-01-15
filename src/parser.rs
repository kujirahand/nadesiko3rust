use crate::tokenizer::{Token, TokenKind};
use crate::tokencur::TokenCur;

#[derive(Debug,Clone)]
pub struct ParseError {
    pub message: String,
    pub line: u32,
    pub fileno: u32,
}
impl ParseError {
    pub fn new(message: String, line: u32, fileno: u32) -> ParseError {
        Self {
            message,
            line,
            fileno
        }
    }
}

#[derive(Debug,Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub line: u32,
    pub fileno: u32,
}
impl Node {
    pub fn new(kind: NodeKind, line: u32, fileno: u32) -> Self {
        Self{ kind, line, fileno }
    }
}

#[derive(Debug,Clone)]
pub enum NodeKind {
    Empty,
    Comment(String),
    List(Vec<Node>),
    Int(isize),
    Number(f64),
}

#[derive(Debug)]
pub struct Parser {
    fileno: u32,
    files: Vec<String>,
    pub nodes: Vec<Node>,
    cur: TokenCur,
    stack: Vec<Node>,
    errors: Vec<ParseError>,
}
impl Parser {
    pub fn new(tokens: Vec<Token>, filename: &str) -> Self {
        let mut parser = Self {
            fileno: 0,
            files: vec![],
            cur: TokenCur::new(tokens),
            nodes: vec![],
            stack: vec![],
            errors: vec![],
        };
        parser.set_filename(filename);
        parser
    }
    fn has_error(&self) -> bool {
        self.errors.len() > 0
    }
    fn set_filename(&mut self, filename: &str) {
        match self.find_files(filename) {
            Some(fileno) => {
                self.fileno = fileno;
            },
            None => {
                self.fileno = self.files.len() as u32;
                self.files.push(filename.to_string());
            },
        };
    }
    fn find_files(&self, filename: &str) -> Option<u32> {
        for (i, fname) in self.files.iter().enumerate() {
            if fname == filename { return Some(i as u32); }
        }
        None
    }
    pub fn throw_error(&mut self, msg: String, line: u32) {
        let err = ParseError::new(msg, line, self.fileno);
        self.errors.push(err);
    }
    pub fn parse(&mut self) -> bool {
        self.sentence_list()
    }
    fn sentence_list(&mut self) -> bool {
        while self.cur.can_read() {
            if self.has_error() { return false; }
            if self.cur.eq_kind(TokenKind::BlockEnd) { break; }
            let tmp_index = self.cur.index;
            self.sentence();
            if self.cur.index == tmp_index {
                let t = self.cur.peek();
                println!("[parser::system.error](sentence_list):{}(line={})", t, t.line);
                self.cur.next_kind();
            }
        }
        true
    }
    fn sentence(&mut self) -> bool {
        // 「ここまで」があれば抜ける
        if self.cur.eq_kind(TokenKind::BlockEnd) { return false; }
        // 改行を無視
        while self.cur.eq_kind(TokenKind::Eol) {
            self.cur.next_kind();
        }
        // コメント
        if self.check_comment() { return true; }
        if self.check_let() { return true; }
        true
    }
    fn check_comment(&mut self) -> bool {
        if !self.cur.eq_kind(TokenKind::Comment) { return false; }
        let t = self.cur.next();
        let node = Node::new(NodeKind::Comment(t.label), t.line, self.fileno);
        self.nodes.push(node);
        true
    }
    fn check_let(&mut self) -> bool {
        if !self.cur.eq_kinds(&[TokenKind::Word, TokenKind::Eq]) { return false; }
        let word:Token = self.cur.next();
        self.cur.next_kind(); // eq
        if !self.check_value() { // error
            self.throw_error(format!("『{}』の代入文で値がありません。", word.label), word.line);
            return false;
        }
        // todo
        false        
    }
    fn check_value(&mut self) -> bool {
        if self.cur.eq_kind(TokenKind::Int) {
            let t = self.cur.next();
            let i:isize = t.label.parse().unwrap_or(0);
            let node = Node::new(NodeKind::Int(i), t.line, self.fileno);
            self.stack.push(node);
            return true;
        }
        false
    }

}

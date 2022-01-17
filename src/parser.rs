use crate::tokenizer::{self, Token, TokenKind};
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
    pub value: NodeValue,
    pub line: u32,
    pub fileno: u32,
}
impl Node {
    pub fn new(kind: NodeKind, value: NodeValue, line: u32, fileno: u32) -> Self {
        Self {
            kind,
            value,
            line, 
            fileno
        }
    }
}
#[derive(Debug,Clone)]
pub enum NodeValue {
    None,
    S(String),
    I(isize),
    F(f64),
    Nodes(Vec<Node>),
    LetVar(String, Vec<Node>),
}
impl NodeValue {
    pub fn to_string(&self) -> String {
        match self {
            NodeValue::None => String::from("None"),
            NodeValue::S(v) => format!("{}", v),
            NodeValue::I(v) => format!("{}", v),
            NodeValue::F(v) => format!("{}", v),
            _ => String::from(""),
        }
    }
}


#[derive(Debug,PartialEq,Clone,Copy)]
pub enum NodeKind {
    Empty,
    Comment,
    List,
    Int,
    Number,
    String,
    StringEx,
    Let,
}

#[derive(Debug)]
pub struct Parser {
    fileno: u32,
    files: Vec<String>,
    pub nodes: Vec<Node>,
    cur: TokenCur,
    stack: Vec<Node>,
    errors: Vec<ParseError>,
    error_count: usize,
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
            error_count: 0,
        };
        parser.set_filename(filename);
        parser
    }
    // for error
    fn has_error(&self) -> bool {
        self.error_count > 0
    }
    pub fn throw_error(&mut self, msg: String, line: u32) {
        let err = ParseError::new(msg, line, self.fileno);
        self.errors.push(err);
        self.error_count += 1;
    }
    pub fn throw_error_token(&mut self, msg: &str, t: Token) {
        let message = format!("『{}』の近くで、{}。", t.label, msg);
        self.throw_error(message, t.line);
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
        let node = Node::new(NodeKind::Comment, NodeValue::S(t.label), t.line, self.fileno);
        self.nodes.push(node);
        true
    }
    fn check_let(&mut self) -> bool {
        if !self.cur.eq_kinds(&[TokenKind::Word, TokenKind::Eq]) { return false; }
        let word: Token = self.cur.next();
        self.cur.next_kind(); // eq
        if !self.check_value() { // error
            self.throw_error(format!("『{}』の代入文で値がありません。", word.label), word.line);
            return false;
        }
        let value = match self.stack.pop() {
            None => {
                self.throw_error(format!("『{}』の代入文で値がありません。", word.label), word.line);
                return false;    
            },
            Some(node) => node,
        };
        let let_node = Node::new(
            NodeKind::Let, 
            NodeValue::LetVar(word.label, vec![value]),
            word.line,
            self.fileno);
        self.nodes.push(let_node);
        // todo: 配列
        false        
    }
    fn check_value(&mut self) -> bool {
        if self.cur.eq_kind(TokenKind::Int) {
            let t = self.cur.next();
            let i:isize = t.label.parse().unwrap_or(0);
            let node = Node::new(NodeKind::Int, NodeValue::I(i), t.line, self.fileno);
            self.stack.push(node);
            self.check_calc_flag();
            return true;
        }
        if self.cur.eq_kind(TokenKind::Number) {
            let t = self.cur.next();
            let v:f64 = t.label.parse().unwrap_or(0.0);
            let node = Node::new(NodeKind::Int, NodeValue::F(v), t.line, self.fileno);
            self.stack.push(node);
            self.check_calc_flag();
            return true;
        }
        if self.cur.eq_kind(TokenKind::String) {
            let t = self.cur.next();
            let node = Node::new(NodeKind::String, NodeValue::S(t.label), t.line, self.fileno);
            self.stack.push(node);
            // todo flag
            return true;
        }
        false
    }
    fn check_calc_flag(&mut self) -> bool {
        // todo: check calc flag
        false
    }

}

#[cfg(test)]
mod test_parser {
    use super::*;
    #[test]
    fn test_parser_comment() {
        let t = tokenizer::tokenize("/*cmt*/");
        let mut p = Parser::new(t, "hoge.nako3");
        assert_eq!(p.parse(), true);
        let node = &p.nodes[0];
        assert_eq!(node.kind, NodeKind::Comment);
        assert_eq!(node.value.to_string(), String::from("cmt"));
    }
    #[test]
    fn test_parser_let() {
        let t = tokenizer::tokenize("aaa = 30");
        let mut p = Parser::new(t, "hoge.nako3");
        assert_eq!(p.parse(), true);
        let node = &p.nodes[0];
        assert_eq!(node.kind, NodeKind::Let);
        let let_value = match &node.value {
            NodeValue::LetVar(var, nodes) => {
                assert_eq!(*var, "aaa".to_string());
                let node = &nodes[0];
                match node.value {
                    NodeValue::I(v) => v,
                    _ => 0,
                }
            },
            _ => 0,
        };
        assert_eq!(let_value, 30);
    }
}
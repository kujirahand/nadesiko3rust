use crate::tokenizer::Token;
use crate::tokencur::TokenCur;

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
}

#[derive(Debug)]
pub struct Parser {
    fileno: usize,
    files: Vec<String>,
    nodes: Vec<Node>,
    cur: TokenCur,
}
impl Parser {
    pub fn new(tokens: Vec<Token>, filename: &str) -> Self {
        let mut parser = Self {
            fileno: 0,
            files: vec![],
            cur: TokenCur::new(tokens),
            nodes: vec![],
        };
        parser.set_filename(filename);
        parser
    }
    fn set_filename(&mut self, filename: &str) {
        match self.find_files(filename) {
            Some(fileno) => {
                self.fileno = fileno;
            },
            None => {
                self.fileno = self.files.len();
                self.files.push(filename.to_string());
            },
        };
    }
    fn find_files(&self, filename: &str) -> Option<usize> {
        for (i, fname) in self.files.iter().enumerate() {
            if fname == filename { return Some(i); }
        }
        None
    }
    pub fn parse() {
        // self.sentences()
    }
}

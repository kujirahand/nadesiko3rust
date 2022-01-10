use crate::tokenizer::{Token, TokenInfo};

#[derive(Debug,Clone)]
pub struct NodeInfo {
    pub fileno: u32,
    pub line: u32,
}
impl NodeInfo {
    pub fn new(line: u32, fileno: u32) -> Self {
        Self{ line, fileno }
    }
}

#[derive(Debug,Clone)]
pub enum Node {
    Nil,
    Comment(NodeInfo, String),
    List(NodeInfo, Vec<Node>),
}
impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Node::Comment(_, label) => write!(f, "Comment:{}", label),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(Debug,Clone)]
pub struct Parser {
    index: usize,
    fileno: usize,
    files: Vec<String>,
    tokens: Vec<Token>,
    top_node: Node,
    last_node: Node,
}
impl Parser {
    pub fn new() -> Self {
        Self {
            index: 0,
            fileno: 0,
            files: vec![],
            tokens: vec![],
            top_node: Node::Nil,
            last_node: Node::Nil,
        }
    }
    pub fn parse(&mut self, tokens: Vec<Token>, filename: &str) -> Result<Node, &str> {
        self.index = 0;
        self.set_filename(filename);
        self.tokens = tokens;
        self.sentences()
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
    fn can_read(&self) -> bool {
        return self.index < self.tokens.len();
    }
    fn peek(&self) -> Token {
        self.tokens[self.index].clone()
    }
    fn sentences(&mut self) -> Result<Node, &str> {
        let list_node: Vec<Node> = vec![];
        Ok(Node::List(NodeInfo::new(0, 0), list_node))
    }
    fn sentence(&mut self) -> Result<Node, &str> {
        Err("todo")
    }
}






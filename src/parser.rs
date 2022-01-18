use crate::tokenizer::{self, Token, TokenKind};
use crate::tokencur::TokenCur;
use crate::node::*;

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
    pub fn to_string(&self) -> String {
        format!("({}){}", self.line, self.message)
    } 
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
        println!("[ERROR] {}", &err.to_string());
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

        // トークンの連続＋命令の場合
        while self.cur.can_read() {
            if self.cur.eq_kind(TokenKind::Eol) { break; }
            if self.check_debug_print() { return true; }
            if !self.check_value() { break; }
        }
        // スタックの余剰があればエラー
        //todo
        true
    }
    fn check_comment(&mut self) -> bool {
        if !self.cur.eq_kind(TokenKind::Comment) { return false; }
        let t = self.cur.next();
        let node = Node::new(NodeKind::Comment, NodeValue::S(t.label), t.line, self.fileno);
        self.nodes.push(node);
        true
    }
    fn check_debug_print(&mut self) -> bool {
        if !self.cur.eq_kind(TokenKind::DebugPrint) { return false; }
        println!("{:?}", self.nodes);
        let print_tok = self.cur.next();
        if !self.stack.len() == 0 {
            self.throw_error_token("『デバッグ表示』で引数がありません。", print_tok);
            return false;
        }
        let value:Node = self.stack.pop().unwrap_or(Node::new_nop());
        let node = Node::new(NodeKind::DebugPrint, NodeValue::Nodes(vec![value]), print_tok.line, self.fileno);
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
            NodeValue::LetVar(NodeValueLet::new(word.label, vec![value])),
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
    fn test_parser_print() {
        let t = tokenizer::tokenize("123をデバッグ表示");
        let mut p = Parser::new(t, "hoge.nako3");
        assert_eq!(p.parse(), true);
        if p.nodes.len() > 0 {
            let node = &p.nodes[0];
            assert_eq!(node.kind, NodeKind::DebugPrint);
            let arg0:String = match &node.value {
                NodeValue::Nodes(nodes) => {
                    if nodes.len() > 0 {
                        nodes[0].value.to_string()
                    } else {
                        "".to_string()
                    }
                },
                _ => String::from(""),
            };
            assert_eq!(arg0, String::from("123"));
        } else {
            assert_eq!("デバッグ表示", "");
        }
    }

    #[test]
    fn test_parser_let() {
        let t = tokenizer::tokenize("aaa = 30");
        let mut p = Parser::new(t, "hoge.nako3");
        assert_eq!(p.parse(), true);
        let node = &p.nodes[0];
        assert_eq!(node.kind, NodeKind::Let);
        let let_value = match &node.value {
            NodeValue::LetVar(v) => {
                assert_eq!(*v.varname, "aaa".to_string());
                let node = &v.value_node[0];
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
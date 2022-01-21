use crate::token::*;
use crate::node::*;
use crate::context::*;
use crate::tokencur::TokenCur;
use crate::operator;
use crate::josi_list;

pub struct Parser {
    pub nodes: Vec<Node>,
    pub context: NodeContext,
    cur: TokenCur,
    fileno: u32,
    stack: Vec<Node>,
}
impl Parser {
    pub fn new() -> Self {
        Self {
            fileno: 0,
            cur: TokenCur::new(vec![]), // dummy
            nodes: vec![],
            stack: vec![],
            context: NodeContext::new(),
        }
    }
    // for error
    pub fn has_error(&self) -> bool {
        self.context.has_error()
    }
    pub fn get_error_str(&self) -> String {
        self.context.get_error_str()
    }
    pub fn throw_error(&mut self, msg: String, line: u32) {
        self.context.throw_error(NodeErrorKind::ParserError, NodeErrorLevel::Error, msg, line, self.fileno);
    }
    pub fn throw_error_token(&mut self, msg: &str, t: Token) {
        let message = format!("『{}』の近くで、{}。", t.label, msg);
        self.throw_error(message, t.line);
    }

    //-------------------------------------------------------------
    // parse
    //-------------------------------------------------------------
    pub fn parse(&mut self, tokens: Vec<Token>, filename: &str) -> Result<Vec<Node>, String> {
        self.cur = TokenCur::new(tokens);
        self.fileno = self.context.set_filename(filename);
        self.get_sentence_list()
    }

    fn get_sentence_list(&mut self) -> Result<Vec<Node>, String> {
        let mut tmp = vec![];
        std::mem::swap(&mut self.nodes, &mut tmp);
        let mut last_index = 0;
        while self.cur.can_read() {
            if self.has_error() { return Err(self.get_error_str()); }
            if self.cur.eq_kind(TokenKind::BlockEnd) { break; }
            self.sentence();
            if self.cur.index == last_index {
                let t = self.cur.peek();
                println!("[error](sentence_list):{}(line={})", t, t.line);
                self.cur.next_kind();
            }
            last_index = self.cur.index;
        }
        std::mem::swap(&mut self.nodes, &mut tmp);
        Ok(tmp)
    }


    fn stack_last_eq(&self, kind: NodeKind) -> bool {
        if self.stack.len() == 0 { return false; }
        let last_node = &self.stack[self.stack.len() - 1];
        return last_node.kind == kind;
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
        // もし文
        if self.check_if() { return true; }
        // トークンの連続＋命令の場合
        while self.cur.can_read() {
            if self.cur.eq_kind(TokenKind::Eol) { break; }
            if !self.check_value() { break; }
            // call function?
            if self.stack_last_eq(NodeKind::CallSysFunc) {
                let callfunc = self.stack.pop().unwrap();
                // println!("@@@sentence@@@{:?}", callfunc);
                self.nodes.push(callfunc);
                return true;
            }
        }
        for n in self.stack.iter() {
            println!("@stack@:{:?}", n);
        }
        // スタックの余剰があればエラー
        //todo
        true
    }

    fn check_comment(&mut self) -> bool {
        if !self.cur.eq_kind(TokenKind::Comment) { return false; }
        let t = self.cur.next();
        let node = Node::new(NodeKind::Comment, NodeValue::S(t.label), None, t.line, self.fileno);
        self.nodes.push(node);
        true
    }

    fn get_if_cond(&mut self, mosi_t: Token) -> Option<Node> {
        // 条件式を取得
        if !self.check_value() {
            self.throw_error_token("『もし』文で条件式がありません。", mosi_t);
            return None;
        }
        // 条件を確認
        let mut cond1 = self.stack.pop().unwrap_or(Node::new_nop());
        let josi1 = cond1.get_josi_str();
        match josi_list::is_josi_mosi(&josi1) {
            Some(active) => {
                if !active {
                    // 否定形にする
                    cond1 = Node::new_operator('!', cond1, Node::new_nop(), mosi_t.line, self.fileno);
                    cond1.josi = Some(josi1.clone());
                }
            },
            None => {
                // 条件が(cond1 が cond2)の場合
                if josi1.eq("が") || josi1.eq("は") {
                    if !self.check_value() {
                        self.throw_error_token("『もし(比較式)ならば』と記述する必要があります。", mosi_t);
                        return None;
                    }
                    let cond2 = self.stack.pop().unwrap_or(Node::new_nop());
                    let josi2 = cond2.get_josi_str();
                    match josi_list::is_josi_mosi(&josi2) {
                        Some(active) => {
                            cond1 = Node::new_operator(if active {'='} else {'!'}, cond1, cond2, mosi_t.line, self.fileno);
                            cond1.josi = Some(josi2);
                        },
                        None => {
                            self.throw_error_token("『もし(値1)が(値2)ならば』と記述する必要があります。", mosi_t);
                            return None;
                        }
                    }
                } else {
                    self.throw_error_token("『もし(比較式)ならば』と記述する必要があります。", mosi_t);
                    return None;
                }
            }
        }
        Some(cond1)
    }
    fn check_if(&mut self) -> bool {
        if !self.cur.eq_kind(TokenKind::If) { return false; }
        let mosi_t = self.cur.next(); // もし
        let cond = match self.get_if_cond(mosi_t) {
            Some(n) => n,
            None => return false,
        };
        // コメントがあれば飛ばす
        while self.cur.eq_kind(TokenKind::Comment) { self.cur.next(); }
        // 単文か複文か
        let multi_sentence = self.cur.eq_kind(TokenKind::Eol);
        if multi_sentence {
            // TODO
        }
        // TODO
        false
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
        // 既に存在する変数への代入?
        let var_name = &word.label;
        let var_info = match self.context.find_var_info(var_name) {
            Some(info) => info,
            None => {
                let new_value = NodeValue::Empty;
                let mut scope = self.context.scopes.pop().unwrap_or(NodeScope::new());
                let no = scope.set_var(var_name, new_value);
                self.context.scopes.push(scope);
                NodeVarInfo{
                    name: Some(String::from(var_name)),
                    level: self.context.scopes.len() - 1,
                    no,
                }
            }
        };
        let node_value_let = NodeValueLet {
            var_name: word.label,
            var_info: var_info,
            value_node: vec![value],
        };
        let let_node = Node::new(
            NodeKind::Let, 
            NodeValue::LetVar(node_value_let),
            None, word.line, self.fileno);
        self.nodes.push(let_node);
        // todo: 配列
        false        
    }

    fn check_paren(&mut self) -> bool {
        // ( ... ) の場合
        if !self.cur.eq_kind(TokenKind::ParenL) { return false; }
        
        let t = self.cur.next(); // skip '('
        if !self.check_value() {
            self.throw_error_token("『(..)』の内側に値が必要です。", t);
            return false;
        }
        // 閉じ括弧まで繰り返し値を読む
        while self.cur.can_read() {
            if self.cur.eq_kind(TokenKind::ParenR) { break; }
            if !self.check_value() {
                self.throw_error_token("『)』閉じカッコが見当たりません。", t);
                return false;
            }
            if self.stack_last_eq(NodeKind::CallSysFunc) { break; }
        }
        let value_node = self.stack.pop().unwrap_or(Node::new_nop());
        if !self.cur.eq_kind(TokenKind::ParenR) {
            self.throw_error_token("『)』閉じカッコが必要です。", t);
            return false;
        }
        let t_close = self.cur.next(); // skip ')'
        let mut node = Node::new_operator('(', value_node, Node::new_nop(), t.line, self.fileno);
        node.josi = t_close.josi;
        self.stack.push(node);
        self.check_operator();
        return true;
    }

    fn check_value(&mut self) -> bool {
        if self.check_paren() {
            return true;
        }
        if self.cur.eq_kind(TokenKind::Int) {
            let t = self.cur.next();
            let i:isize = t.label.parse().unwrap_or(0);
            let node = Node::new(NodeKind::Int, NodeValue::I(i), t.josi, t.line, self.fileno);
            self.stack.push(node);
            self.check_operator();
            return true;
        }
        if self.cur.eq_kind(TokenKind::Number) {
            let t = self.cur.next();
            let v:f64 = t.label.parse().unwrap_or(0.0);
            let node = Node::new(NodeKind::Int, NodeValue::F(v), t.josi, t.line, self.fileno);
            self.stack.push(node);
            self.check_operator();
            return true;
        }
        if self.cur.eq_kind(TokenKind::String) || self.cur.eq_kind(TokenKind::StringEx) {
            let t = self.cur.next();
            let node = Node::new(NodeKind::String, NodeValue::S(t.label), t.josi, t.line, self.fileno);
            self.stack.push(node);
            self.check_operator_str();
            return true;
        }
        if self.check_variable() {
            return true;
        }
        false
    }

    fn check_call_function(&mut self, node: &mut Node) -> bool {
        let var_info = match &node.value {
            NodeValue::GetVar(info) => info,
            _ => return false,
        };
        let val = self.context.get_var_value(&var_info).unwrap();
        match val {
            NodeValue::SysFunc(no, _) => {
                node.kind = NodeKind::CallSysFunc;
                let mut arg_nodes = vec![];
                let info:&SysFuncInfo = &self.context.sysfuncs[no];
                // todo: 助詞を確認する
                // 引数の数だけstackからpopする
                for _arg in info.args.iter() {
                    let n = self.stack.pop().unwrap_or(Node::new_nop());
                    arg_nodes.push(n);
                }
                node.value = NodeValue::SysFunc(no, arg_nodes);
                return true;
            },
            _ => {},    
        }
        false
    }

    fn check_variable(&mut self) -> bool {
        if !self.cur.eq_kind(TokenKind::Word) { return false; }
        // 変数を得る
        let mut node = self.get_variable();
        if self.check_call_function(&mut node) {
            self.stack.push(node);
            return true;
        }
        self.stack.push(node);
        self.check_operator();
        return true;
    }
    fn get_variable(&mut self) -> Node {
        let t = self.cur.next(); // 変数名
        let var_name = String::from(t.label);
        let var_info = match self.context.find_var_info(&var_name) {
            Some(mut i) => {
                i.name = Some(var_name);
                i
            },
            None => {
                let level = self.get_scope_level();
                let no = self.context.scopes[level].set_var(&var_name, NodeValue::Empty);
                NodeVarInfo {
                    name: Some(var_name), 
                    level,
                    no
                }
            },
        };
        let node = Node::new(NodeKind::GetVar, NodeValue::GetVar(var_info), t.josi, t.line, self.fileno);
        node
    }
    fn get_scope_level(&self) -> usize {
        self.context.scopes.len() - 1
    }
    fn check_operator(&mut self) -> bool {
        if self.cur.eq_operator() {
            let op_t = self.cur.next();
            let cur_flag = op_t.as_char();
            if !self.check_value() {
                self.throw_error_token(&format!("演算子『{}』の後に値がありません。", op_t.label), op_t);
                return false;
            }
            // a + (b + c)
            let value_bc = self.stack.pop().unwrap_or(Node::new_nop());
            let value_a  = self.stack.pop().unwrap_or(Node::new_nop());
            // 演算子の順序を確認
            let pri_cur  = operator::get_priority(cur_flag);
            let pri_next = operator::get_node_priority(&value_bc);
            // [a +] [b * c] = priority[<] 入れ替えなし
            // [a *] [b + c] = priority[>] 入れ替えあり => [a * b] + c
            if pri_cur > pri_next {
                // 入れ替え
                match value_bc.value {
                    NodeValue::Operator(mut op) => {
                        let value_c = op.nodes.pop().unwrap();
                        let value_b = op.nodes.pop().unwrap();
                        let new_node = Node::new_operator(
                            op.flag,
                            Node::new_operator(cur_flag, value_a, value_b, value_bc.line, value_bc.fileno),
                            value_c,
                            value_bc.line, value_bc.fileno);
                        self.stack.push(new_node);
                    },
                    _ => { self.throw_error_token("システムエラー::演算子", op_t); return false; }
                }
                return true;
            }
            // 入れ替えなし              
            let op_node = Node::new_operator(
                cur_flag, value_a, value_bc, 
                op_t.line, self.fileno
            );
            self.stack.push(op_node);
            return true;
        }
        false
    }
    fn check_operator_str(&mut self) -> bool {
        if self.cur.eq_operator_str() {
            let op_t = self.cur.next();
            let cur_flag = op_t.as_char();
            if !self.check_value() {
                self.throw_error_token(&format!("演算子『{}』の後に値がありません。", op_t.label), op_t);
                return false;
            }
            // a & b
            let value_b = self.stack.pop().unwrap_or(Node::new_nop());
            let value_a  = self.stack.pop().unwrap_or(Node::new_nop());
            // 文字列に関しては式の入れ替え不要              
            let op_node = Node::new_operator(
                cur_flag, value_a, value_b, 
                op_t.line, self.fileno
            );
            self.stack.push(op_node);
            return true;
        }
        false
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;
    use crate::tokenizer::tokenize;

    #[test]
    fn test_parser_comment() {
        let t = tokenize("/*cmt*/");
        let mut p = Parser::new();
        let nodes = p.parse(t, "hoge.nako3").unwrap();
        let node = &nodes[0];
        assert_eq!(node.kind, NodeKind::Comment);
        assert_eq!(node.value.to_string(), String::from("cmt"));
    }

    #[test]
    fn test_parser_let() {
        let t = tokenize("aaa = 30");
        let mut p = Parser::new();
        let nodes = p.parse(t, "hoge.nako3").unwrap(); 
        let node = &nodes[0];
        assert_eq!(node.kind, NodeKind::Let);
        let let_value = match &node.value {
            NodeValue::LetVar(v) => {
                assert_eq!(*v.var_name, "aaa".to_string());
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
use crate::token::*;
use crate::node::*;
use crate::context::*;
use crate::tokencur::TokenCur;
use crate::operator;
use crate::josi_list;

pub struct Parser {
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
        let mut nodes: Vec<Node> = vec![];
        // 文を繰り返し得る
        while self.cur.can_read() {
            // 文の連続の終了条件
            if self.has_error() { return Err(self.get_error_str()); }
            if self.cur.eq_kind(TokenKind::BlockEnd) { break; }
            if self.cur.eq_kind(TokenKind::Else) { break; }
            // 連続で文を読む
            if let Some(node) = self.sentence() {
                nodes.push(node);
                continue;
            }
            break;
        }
        Ok(nodes)
    }

    fn stack_last_eq(&self, kind: NodeKind) -> bool {
        if self.stack.len() == 0 { return false; }
        let last_node = &self.stack[self.stack.len() - 1];
        return last_node.kind == kind;
    }

    fn sentence(&mut self) -> Option<Node> {
        // 「ここまで」があれば抜ける
        if self.cur.eq_kind(TokenKind::BlockEnd) { return None; }
        // 「違えば」があれば抜ける
        if self.cur.eq_kind(TokenKind::Else) { return None; }
        // 改行を無視
        while self.cur.eq_kind(TokenKind::Eol) {
            self.cur.next_kind();
        }
        // コメント
        if let Some(node) = self.check_comment() { return Some(node); }
        if let Some(node) = self.check_let() { return Some(node); }
        // もし文
        if let Some(node) = self.check_if() { return Some(node); }
        // トークンの連続＋命令の場合
        while self.cur.can_read() {
            if self.cur.eq_kind(TokenKind::Eol) { break; }
            if !self.check_value() { break; }
            // call function?
            if self.stack_last_eq(NodeKind::CallSysFunc) {
                let callfunc = self.stack.pop().unwrap();
                // println!("@@@sentence@@@{:?}", callfunc);
                return Some(callfunc);
            }
        }
        // スタックの余剰があればエラー
        for n in self.stack.iter() {
            println!("@@TODO:スタックの余剰:::stack@:{:?}", n);
        }
        //todo
        None
    }

    fn check_comment(&mut self) -> Option<Node> {
        if !self.cur.eq_kind(TokenKind::Comment) { return None; }
        let t = self.cur.next();
        let node = Node::new(NodeKind::Comment, NodeValue::S(t.label), None, t.line, self.fileno);
        Some(node)
    }

    fn check_if_cond(&mut self, mosi_t: Token) -> Option<Node> {
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
                    cond1 = Node::new_operator('!', cond1, Node::new_nop(), Some(josi1), mosi_t.line, self.fileno);
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
                            cond1 = Node::new_operator(if active {'='} else {'≠'}, cond1, cond2, Some(josi2), mosi_t.line, self.fileno);
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

    fn skip_eol_comment(&mut self) {
        while self.cur.can_read() {
            if self.cur.eq_kind(TokenKind::Comment) {
                self.cur.next();
                continue;
            }
            if self.cur.eq_kind(TokenKind::Eol) {
                self.cur.next();
                continue;
            }
            break;
        }
    }

    fn check_if(&mut self) -> Option<Node> {
        // 「もし」があるか？
        if !self.cur.eq_kind(TokenKind::If) {
            return None;
        }
        let mosi_t = self.cur.next(); // もし
        if self.cur.eq_kind(TokenKind::Comma) {
            self.cur.next();
        }
        let cond = match self.check_if_cond(mosi_t) {
            Some(n) => n,
            None => return None,
        };
        // コメントがあれば飛ばす
        while self.cur.eq_kind(TokenKind::Comment) { self.cur.next(); }
        let mut true_nodes: Vec<Node> = vec![];
        let mut false_nodes: Vec<Node> = vec![];
        // 真ブロックの取得 --- 単文か複文か
        let mut single_sentence = true;
        if self.cur.eq_kind(TokenKind::BlockBegin) {
            single_sentence = false;
            self.cur.next(); // skip ここから
        }
        if self.cur.eq_kind(TokenKind::Eol) {
            single_sentence = false;
            self.cur.next(); // skip EOL
        }
        if single_sentence {
            if let Some(node) = self.sentence() {
                true_nodes = vec![node];
            }
            // （コメント）＋（一度だけの改行）を許容
            while self.cur.eq_kind(TokenKind::Comment) { self.cur.next(); }
            if self.cur.eq_kind(TokenKind::Eol) { self.cur.next(); }
        } else {
            // multi sentences
            if let Ok(nodes) = self.get_sentence_list() {
                true_nodes = nodes;
            }
            self.skip_eol_comment();
        }
        // 偽ブロックの取得 --- 単文か複文か
        if self.cur.eq_kind(TokenKind::Else) {
            self.cur.next(); // skip 違えば
            while self.cur.eq_kind(TokenKind::Comment) { self.cur.next(); }
            single_sentence = true;
            if self.cur.eq_kind(TokenKind::Eol) {
                single_sentence = false;
                self.cur.next(); // skip Eol
            }
            if self.cur.eq_kind(TokenKind::BlockBegin) {
                single_sentence = false;
                self.cur.next(); // skip ここから
            }
            if single_sentence {
                if let Some(node) = self.sentence() {
                    false_nodes = vec![node];
                }
            } else {
                if let Ok(nodes) = self.get_sentence_list() {
                    false_nodes = nodes;
                }
            }
        }
        // nodes -> node
        let t_node = Node::new(NodeKind::NodeList, NodeValue::NodeList(true_nodes), None, 0, self.fileno);
        let f_node = Node::new(NodeKind::NodeList, NodeValue::NodeList(false_nodes), None, 0, self.fileno);
        let if_node = Node::new(NodeKind::If, NodeValue::NodeList(vec![cond, t_node, f_node]), None, 0, self.fileno);
        Some(if_node)
    }

    fn check_let(&mut self) -> Option<Node> {
        if !self.cur.eq_kinds(&[TokenKind::Word, TokenKind::Eq]) { return None; }
        let word: Token = self.cur.next();
        self.cur.next_kind(); // eq
        if !self.check_value() { // error
            self.throw_error(format!("『{}』の代入文で値がありません。", word.label), word.line);
            return None;
        }
        let value = match self.stack.pop() {
            None => {
                self.throw_error(format!("『{}』の代入文で値がありません。", word.label), word.line);
                return None;
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
        Some(let_node)
        // todo: 配列
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
        let node = Node::new_operator('(', value_node, Node::new_nop(), t_close.josi, t.line, self.fileno);
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
        if self.cur.eq_kind(TokenKind::True) || self.cur.eq_kind(TokenKind::False) {
            let t = self.cur.next(); // 値
            let b: bool = if t.label.eq("真") { true } else { false };
            let node = Node::new(NodeKind::Bool, NodeValue::B(b), t.josi, t.line, self.fileno);
            self.stack.push(node);
            self.check_operator();
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
            // (a + b) [+] c
            let op_t = self.cur.next();
            let cur_flag = op_t.as_char();
            // (a + b) + [c]
            if !self.check_value() {
                self.throw_error_token(&format!("演算子『{}』の後に値がありません。", op_t.label), op_t);
                return false;
            }
            let value_c = self.stack.pop().unwrap_or(Node::new_nop());
            let c_josi = value_c.josi.clone();
            let value_ab  = self.stack.pop().unwrap_or(Node::new_nop());
            // 演算子の順序を確認
            let pri_cur  = operator::get_priority(cur_flag);
            let pri_prev = operator::get_node_priority(&value_ab);
            // (a * b) [+] c = priority[現在 > 前回] 入れ替えなし
            // (a + b) [*] c = priority[現在 < 前回] 入れ替えあり => a + (b * c)
            if pri_cur > pri_prev {
                // 入れ替え
                match value_ab.value {
                    NodeValue::Operator(mut op) => {
                        let value_b = op.nodes.pop().unwrap();
                        let value_a = op.nodes.pop().unwrap();
                        let new_node = Node::new_operator(
                            op.flag,
                            value_a,
                            Node::new_operator(cur_flag, value_b, value_c, None, value_ab.line, value_ab.fileno),
                            c_josi,
                            op_t.line, self.fileno);
                        self.stack.push(new_node);
                    },
                    _ => { self.throw_error_token("システムエラー::演算子", op_t); return false; }
                }
                return true;
            }
            // 入れ替えなし              
            let op_node = Node::new_operator(
                cur_flag, value_ab, value_c,
                c_josi, 
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
            let josi_b = value_b.josi.clone();
            let value_a  = self.stack.pop().unwrap_or(Node::new_nop());
            // 文字列に関しては式の入れ替え不要              
            let op_node = Node::new_operator(
                cur_flag, value_a, value_b, 
                josi_b, op_t.line, self.fileno
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
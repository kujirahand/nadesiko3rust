use crate::token::*;
use crate::node::*;
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
        // 最初に関数定義があるかどうか調べる
        self.pre_read_def_func();
        // 冒頭から改めて読む
        self.cur.index = 0;
        self.get_sentence_list()
    }

    fn pre_read_def_func(&mut self) {
        // 関数定義だけを先読みする
        while self.cur.can_read() {
            if self.cur.eq_kind(TokenKind::DefFunc) {
                self.check_def_func(true);
                continue;
            }
            self.cur.next();
        }
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

    fn stack_last_josi_eq(&self, josi_s: &str) -> bool {
        if self.stack.len() == 0 { return false; }
        let last_node = &self.stack[self.stack.len() - 1];
        last_node.eq_josi(josi_s)
    }

    fn new_simple_node(&self, kind: NodeKind, t: &Token) -> Node {
        return Node::new(kind, NodeValue::Empty, None, t.line, self.fileno)
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
        // 関数定義
        if self.cur.eq_kind(TokenKind::DefFunc) { return self.check_def_func(false); }
        // 抜ける・続ける・戻る
        if self.cur.eq_kind(TokenKind::Break) {
            let t = self.cur.next();
            return Some(self.new_simple_node(NodeKind::Break, &t));
        }
        if self.cur.eq_kind(TokenKind::Continue) {
            let t = self.cur.next();
            return Some(self.new_simple_node(NodeKind::Continue, &t));
        }
        // トークンの連続＋命令の場合
        while self.cur.can_read() {
            if self.cur.eq_kind(TokenKind::Eol) { break; }
            if !self.check_value() { break; }
            // call function?
            if self.stack_last_eq(NodeKind::CallSysFunc) || self.stack_last_eq(NodeKind::CallUserFunc) {
                let callfunc = self.stack.pop().unwrap();
                return Some(callfunc);
            }
            if self.cur.eq_kind(TokenKind::Kai) {
                return self.check_kai();
            }
            if self.cur.eq_kind(TokenKind::For) {
                return self.check_for();
            }
            if self.cur.eq_kind(TokenKind::Return) {
                let ret_t = self.cur.next();
                let mut arg = vec![];
                if self.stack_last_josi_eq("で") {
                    arg.push(self.stack.pop().unwrap_or(Node::new_nop()));
                }
                return Some(Node::new(NodeKind::Return, NodeValue::NodeList(arg), None, ret_t.line, self.fileno));
            }
        }
        // スタックの余剰があればエラーとして報告する
        if self.stack.len() > 0 {
            let mut line = 0;
            let mut errmsg = String::from("計算式に次の余剰があります。\n");
            while let Some(n) = self.stack.pop() {
                errmsg = format!(
                    "{}({})\n",
                    errmsg,
                    n.to_string());
                line = n.line;
            }
            self.throw_error(format!("{}必要なら式を(式)のようにカッコで囲ってみてください。", errmsg), line);
        }
        None
    }

    fn check_for(&mut self) -> Option<Node> {
        let kai_t = self.cur.next(); // skip 繰り返す
        
        // [Iを][0から][9まで]繰り返す
        let made_node = self.stack.pop().unwrap_or(Node::new_nop());
        let kara_node = self.stack.pop().unwrap_or(Node::new_nop());
        let loop_node = if self.stack_last_josi_eq("を") || self.stack_last_josi_eq("で") {
            self.stack.pop().unwrap_or(Node::new_nop())
        } else {
            Node::new_nop()
        };

        // 繰り返す内容
        self.skip_comma_comment();
        let mut single_sentence = true;
        if self.cur.eq_kind(TokenKind::BlockBegin) {
            single_sentence = false;
            self.cur.next(); // ここから
        }
        if self.cur.eq_kind(TokenKind::Eol) {
            single_sentence = false;
            self.cur.next(); // LF
        }
        // 内容を取得
        let mut body_nodes = vec![];
        if single_sentence {
            let node = match self.sentence() {
                Some(node) => node,
                None => Node::new_nop(),
            };
            body_nodes.push(node);
        } else {
            body_nodes = match self.get_sentence_list() {
                Ok(nodes) => nodes,
                Err(_) => return None,
            };
            if self.cur.eq_kind(TokenKind::BlockEnd) {
                self.cur.next(); // skip ここまで
            }
        }
        // Nodeを生成して返す
        let for_node = Node::new(
            NodeKind::For, NodeValue::NodeList(vec![
                loop_node,
                kara_node,
                made_node,
                Node::new(NodeKind::NodeList, NodeValue::NodeList(body_nodes), None, kai_t.line, self.fileno),
            ]), None, kai_t.line, self.fileno);
        Some(for_node)
    }

    fn check_kai(&mut self) -> Option<Node> {
        let kai_t = self.cur.next(); // skip 回
        if self.cur.eq_kind(TokenKind::For) {
            self.cur.next(); // skip 繰り返す
        }
        let kaisu_node = self.stack.pop().unwrap_or(Node::new_nop());
        while self.cur.eq_kind(TokenKind::Comment) {
            self.cur.next(); 
        }
        let mut single_sentence = true;
        if self.cur.eq_kind(TokenKind::BlockBegin) {
            single_sentence = false;
            self.cur.next(); // ここから
        }
        if self.cur.eq_kind(TokenKind::Eol) {
            single_sentence = false;
            self.cur.next(); // LF
        }
        let mut body_nodes = vec![];
        if single_sentence {
            let node = match self.sentence() {
                Some(node) => node,
                None => Node::new_nop(),
            };
            body_nodes.push(node);
        } else {
            body_nodes = match self.get_sentence_list() {
                Ok(nodes) => nodes,
                Err(_) => return None,
            };
            if self.cur.eq_kind(TokenKind::BlockEnd) {
                self.cur.next(); // skip ここまで
            }
        }
        let kai_node = Node::new(NodeKind::Kai, NodeValue::NodeList(vec![
            kaisu_node,
            Node::new_node_list(body_nodes, kai_t.line, self.fileno),
        ]), None, kai_t.line, self.fileno);
        Some(kai_node)
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
    fn skip_comma_comment(&mut self) {
        while self.cur.can_read() {
            if self.cur.eq_kind(TokenKind::Comment) {
                self.cur.next();
                continue;
            }
            if self.cur.eq_kind(TokenKind::Comma) {
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
        // 条件式を得る
        let cond = match self.check_if_cond(mosi_t) {
            Some(n) => n,
            None => return None,
        };
        // コメントなどがあれば飛ばす
        self.skip_comma_comment();
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
            self.skip_comma_comment();
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

    fn check_value(&mut self) -> bool {
        if !self.check_value_one() {
            return false;
        }
        let value = match self.stack.last() {
            Some(v) => v,
            None => return false,
        };
        if value.josi == None { // 助詞がなければ続きはない
            return true;
        }
        // 助詞があれば、それは関数の引数なので連続で値を読む
        while self.cur.can_read() {
            if self.check_value() {
                if self.stack_last_eq(NodeKind::CallSysFunc) { return true; }
                if self.stack_last_eq(NodeKind::CallUserFunc) { return true; }
                if self.stack_last_josi_eq("") {
                    break;              
                }
            }
        }
        false
    }

    fn check_let(&mut self) -> Option<Node> {
        if !self.cur.eq_kinds(&[TokenKind::Word, TokenKind::Eq]) { return None; }
        let word: Token = self.cur.next();
        self.cur.next(); // eq

        // 値を取得する
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
        // todo: ローカル変数を実装する
        // todo: 配列の代入
        // グローバル変数への代入
        let var_name = &word.label;
        self.context.scopes.set_value(1, &var_name, NodeValue::Empty);
        let node_value_let = NodeValueLet {
            var_name: word.label,
            value_node: vec![value],
        };
        let let_node = Node::new(
            NodeKind::Let, 
            NodeValue::LetVar(node_value_let),
            None, word.line, self.fileno);
        Some(let_node)
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
            if self.stack_last_eq(NodeKind::CallUserFunc) { break; }
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

    // 値(+演算子)を1つ読む
    fn check_value_one(&mut self) -> bool {
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
        if self.cur.eq_kind(TokenKind::String) {
            let t = self.cur.next();
            let node = Node::new(NodeKind::String, NodeValue::S(t.label), t.josi, t.line, self.fileno);
            self.stack.push(node);
            self.check_operator();
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
        if self.cur.eq_kind(TokenKind::Word) {
            return self.check_variable();
        }
        false
    }

    fn read_func_args(&mut self, func_name: &str, args: Vec<SysArg>, line: u32) -> Vec<Node> {
        let mut arg_nodes = vec![];
        let mut err_msg = String::new();
        // todo: 助詞を確認する
        // 引数の数だけstackからpopする
        for _arg in args.iter() {
            let n = match self.stack.pop() {
                Some(n) => n,
                None => {
                    err_msg = format!("{}『{}』の引数が不足しています。", err_msg, String::from(func_name));
                    Node::new_nop()
                }
            };
            arg_nodes.push(n);
        }
        if err_msg.ne("") {
            self.context.throw_error(
                NodeErrorKind::ParserError, NodeErrorLevel::Error,
                err_msg, line, self.fileno);    
        }
        arg_nodes
    }

    fn check_variable(&mut self) -> bool {
        // 変数を得る
        let word_t = self.cur.next(); // 変数名 || 関数名
        let name = &word_t.label;
        let mut info = match self.context.find_var_info(name) {
            Some(info) => info,
            None => {
                // 変数がなければ作る
                self.context.scopes.set_value_local_scope(name, NodeValue::Empty)
            }
        };
        // 変数か関数か
        let node = match self.context.get_var_value(&info) {
            Some(value) => {
                // メタ情報を得る
                let meta = match self.context.get_var_meta(&info) {
                    Some(v) => v,
                    None => return false,
                };
                // 関数呼び出しかどうか調べる
                match value {
                    // 関数呼び出しノードを作る
                    NodeValue::SysFunc(name, no, _) => {
                        match meta.kind {
                            NodeVarKind::SysFunc(args) => {
                                let nodes = self.read_func_args(&name, args, word_t.line);
                                let sys_func_node = Node::new(
                                    NodeKind::CallSysFunc, NodeValue::SysFunc(name, no, nodes), 
                                    word_t.josi, word_t.line, self.fileno);
                                sys_func_node
                            },
                            NodeVarKind::UserFunc(args) => {
                                let nodes = self.read_func_args(&name, args, word_t.line);
                                let user_func_node = Node::new(
                                    NodeKind::CallUserFunc, NodeValue::SysFunc(name, no, nodes), 
                                    word_t.josi, word_t.line, self.fileno);
                                    user_func_node
                            },
                            _ => return false,
                        }
                    },
                    // 変数の参照
                    _ => {
                        info.name = Some(String::from(name));
                        let var_node = Node::new(
                            NodeKind::GetVar,
                            NodeValue::GetVar(info),
                            word_t.josi, word_t.line, self.fileno);
                        var_node
                    }
                }
            },
            // 絶対ある
            None => { return false },
        };
        self.stack.push(node);
        self.check_operator();
        return true;
    }

    fn check_operator(&mut self) -> bool {
        if !self.cur.eq_operator() { return false; }
        let op_t = self.cur.next();
        let cur_flag = op_t.as_char();
        if !self.check_value() {
            self.throw_error_token(&format!("演算子『{}』の後に値がありません。", op_t.label), op_t);
            return false;
        }
        // a [+] (b + c)
        let value_bc = self.stack.pop().unwrap_or(Node::new_nop());
        let c_josi = value_bc.josi.clone();
        let value_a  = self.stack.pop().unwrap_or(Node::new_nop());
        // println!("@@[a:{} {} bc:{}]", value_a.to_string(), cur_flag, value_bc.to_string());
        // 演算子の順序を確認
        let pri_cur  = operator::get_priority(cur_flag);
        let pri_prev = operator::get_node_priority(&value_bc);
        // println!("cur={}[{}],prev={}[{}]", pri_cur, cur_flag, pri_prev, value_bc.to_string());
        // a + [b * c] = priority[現在 > 前回] 入れ替えなし
        // a * [b + c] = priority[現在 < 前回] 入れ替えあり => (a * b) + c
        if pri_cur > pri_prev {
            // 入れ替え
            match value_bc.value {
                NodeValue::Operator(mut op) => {
                    let value_c = op.nodes.pop().unwrap();
                    let value_b = op.nodes.pop().unwrap();
                    let new_node = Node::new_operator(
                        op.flag,
                        Node::new_operator(cur_flag, value_a, value_b, None, value_c.line, value_c.fileno),
                        value_c,
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
            cur_flag, value_a, value_bc,
            c_josi, 
            op_t.line, self.fileno
        );
        self.stack.push(op_node);
        return true;
    }

    fn read_def_func_arg(&mut self) -> Vec<SysArg> {
        let mut args: Vec<SysArg> = vec![];
        if self.cur.eq_kind(TokenKind::ParenL) {
            self.cur.next(); // skip '('
        }
        while self.cur.can_read() {
            if self.cur.eq_kind(TokenKind::ParenR) {
                self.cur.next(); // skip ')'
                break;
            }
            if !self.cur.eq_kind(TokenKind::Word) {
                self.throw_error_token(&format!("関数の引数定義は語句が必要です。"), self.cur.peek());
                break;
            }
            let w = self.cur.next(); // 語句を1つ得る
            // argsに同じ語句があるか
            let mut flag_reg = false;
            for arg in args.iter_mut() {
                if arg.name == w.label {
                    arg.josi_list.push(w.josi.clone().unwrap_or(String::new()));
                    flag_reg = true;
                }
            }
            if flag_reg == false {
                args.push(SysArg{
                    name: w.label, 
                    josi_list: vec![w.josi.clone().unwrap_or(String::new())]
                });
            }
        }
        args
    }

    fn check_def_func(&mut self, pre_read: bool) -> Option<Node> {
        if !self.cur.eq_kind(TokenKind::DefFunc) { return None; }
        let def_t = self.cur.next();
        // 引数定義を取得 : ●(引数)関数名
        let mut args: Vec<SysArg> = vec![];
        if self.cur.eq_kind(TokenKind::ParenL) {
            args = self.read_def_func_arg();
        }
        // 関数名を取得
        if !self.cur.eq_kind(TokenKind::Word) {
            self.throw_error_token("関数名がありません", def_t);
            return None;
        }
        let name_t = self.cur.next(); // skip name
        let name_s = name_t.label.clone();
        // 旧引数定義方法 : ●関数名(引数)
        if self.cur.eq_kind(TokenKind::ParenL) {
            args = self.read_def_func_arg();
        }
        // 関数を登録
        let scope = &mut self.context.scopes.scopes[1];
        // 変数に名前を登録
        let no = scope.set_var(&name_t.label, NodeValue::SysFunc(name_s.clone(), 0, vec![]));
        let mut meta = &mut scope.var_metas[no];
        meta.kind = NodeVarKind::UserFunc(args.clone());
        meta.read_only = true;
        if pre_read {
            // 本文を見ずに抜ける
            return None;
        }
        // ローカル変数をスコープに追加
        let mut local_scope = NodeScope::new();
        for arg in args.iter() {
            local_scope.set_var(&arg.name, NodeValue::Empty);
        }
        self.context.scopes.push_local(local_scope);
        // 関数本文ブロックを取得
        let body_nodes = match self.get_sentence_list() {
            Ok(nodes) => nodes,
            Err(err) => {
                self.throw_error_token(&format!("関数『{}』の定義でエラー。{}", name_t.label, err), def_t);
                return None;
            },               
        };
        if self.cur.eq_kind(TokenKind::BlockEnd) {
            self.cur.next(); // skip ここまで
        }
        // ローカルスコープから抜ける
        self.context.scopes.pop_local();
        // 関数本体を変数に登録
        let func_value: NodeValue = NodeValue::SysFunc(name_s.clone(), no, body_nodes);
        self.context.scopes.set_value(1, &name_s, func_value);
        Some(Node::new(NodeKind::Comment, NodeValue::S(format!("関数『{}』の定義", name_s)), None, def_t.line, self.fileno))
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
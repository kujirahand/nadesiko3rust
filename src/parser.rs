//! 構文解析器

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
    fn stack_last_josi(&self) -> Option<String> {
        if self.stack.len() == 0 { return None; }
        let last_node = &self.stack[self.stack.len() - 1];
        last_node.josi.clone()
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
        // 代入文
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
            if self.cur.eq_kind(TokenKind::Dainyu) { return self.check_dainyu(); }
            // call function?
            if self.stack_last_eq(NodeKind::CallSysFunc) || self.stack_last_eq(NodeKind::CallUserFunc) {
                let callfunc = self.stack.pop().unwrap_or(Node::new_nop());
                // 連文の「して」がある場合、もう一文読む
                if callfunc.is_renbun_josi() {
                    let t = self.cur.peek();
                    let mut renbun = vec![callfunc];
                    loop {
                        let cur_index = self.cur.index;
                        if !self.check_value() {
                            self.cur.index = cur_index;
                            break;
                        }
                        let callfunc2 = self.stack.pop().unwrap_or(Node::new_nop());
                        let is_renbun = callfunc2.is_renbun_josi();
                        renbun.push(callfunc2);
                        if is_renbun { continue; }
                        break;
                    }
                    return Some(Node::new_node_list(renbun, t.line, self.fileno));
                }
                println!("josi={:?}",callfunc.josi);
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
        let kai_t = self.cur.next(); // skip "回"
        if self.cur.eq_kind(TokenKind::For) {
            self.cur.next(); // skip "繰り返す"
        }
        let kaisu_node = self.stack.pop().unwrap_or(Node::new_nop());
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
        let mosi_line = mosi_t.line;
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
        let mut t_single_sentence = true;
        let mut f_single_sentence = true;
        if self.cur.eq_kind(TokenKind::BlockBegin) {
            t_single_sentence = false;
            self.cur.next(); // skip ここから
        }
        if self.cur.eq_kind(TokenKind::Eol) {
            t_single_sentence = false;
            self.cur.next(); // skip EOL
        }
        // 単文の場合
        if t_single_sentence {
            if let Some(node) = self.sentence() {
                true_nodes = vec![node];
            }
            // （コメント）＋（一度だけの改行）を許容
            while self.cur.eq_kind(TokenKind::Comment) { self.cur.next(); }
            while self.cur.eq_kind(TokenKind::Eol) { self.cur.next(); }
        } else {
            // 複文の場合
            if let Ok(nodes) = self.get_sentence_list() {
                true_nodes = nodes;
            }
            self.skip_eol_comment();
        }
        // 違えば、もし?
        if self.cur.eq_kinds(&[TokenKind::Else, TokenKind::If]) {
            self.cur.next(); // skip 違えば
            let elseif_node = self.check_if().unwrap_or(Node::new_nop());
            false_nodes = vec![elseif_node];
        }
        // 偽ブロックの取得 --- 単文か複文か
        else if self.cur.eq_kind(TokenKind::Else) {
            self.cur.next(); // skip 違えば
            self.skip_comma_comment();
            if self.cur.eq_kind(TokenKind::Eol) {
                f_single_sentence = false;
                self.cur.next(); // skip Eol
            }
            if self.cur.eq_kind(TokenKind::BlockBegin) {
                f_single_sentence = false;
                self.cur.next(); // skip ここから
            }
            if f_single_sentence {
                if let Some(node) = self.sentence() {
                    false_nodes = vec![node];
                }
            } else {
                if let Ok(nodes) = self.get_sentence_list() {
                    false_nodes = nodes;
                }
            }
        }
        if !t_single_sentence || !f_single_sentence {
            if self.cur.eq_kind(TokenKind::BlockEnd) {
                self.cur.next(); // skip ここまで
            }
        }
        // nodes -> node
        let t_node = Node::new(NodeKind::NodeList, NodeValue::NodeList(true_nodes), None, 0, self.fileno);
        let f_node = Node::new(NodeKind::NodeList, NodeValue::NodeList(false_nodes), None, 0, self.fileno);
        let if_node = Node::new(NodeKind::If, NodeValue::NodeList(vec![cond, t_node, f_node]), None, mosi_line, self.fileno);
        Some(if_node)
    }

    fn check_value(&mut self) -> bool {
        // 値を一つ取得
        if !self.check_value_one() {
            return false;
        }
        // 助詞を確認
        let josi_opt = self.stack_last_josi();
        // 助詞がなければ続きはない
        let josi_s = match josi_opt {
            None => return true,
            Some(s) => s,
        };
        // 『もし..ならば』であれば続きは読まない
        if let Some(_) = josi_list::is_josi_mosi(&josi_s) {
            return true;
        }

        // 助詞があれば、それは関数の引数なので連続で値を読む
        while self.cur.can_read() {
            if self.check_value_one() {
                if self.stack_last_eq(NodeKind::CallSysFunc) { return true; }
                if self.stack_last_eq(NodeKind::CallUserFunc) { return true; }
                let josi_opt = self.stack_last_josi();
                let josi_s = match josi_opt {
                    None => return true,
                    Some(s) => s,
                };
                if let Some(_) = josi_list::is_josi_mosi(&josi_s) {
                    return true;
                }
                continue;
            }
            break;
        }
        true
    }

    fn check_let_array(&mut self) -> Option<Node> {
        // --- ここで調べるトークンの並び
        // word [ index ] = value
        // ---
        let old_index = self.cur.index;
        if !self.cur.eq_kinds(&[TokenKind::Word, TokenKind::BracketL]) {
            return None;
        }
        // word "["
        let word_t = self.cur.next();
        let bracket_t = self.cur.next();
        let mut index_vec = vec![];
        loop {
            // index
            let index_b = self.check_value();
            if !index_b {
                let msg = format!("変数『{}』への配列アクセスでインデックスの指定エラー。", word_t.label);
                self.throw_error_token(&msg, bracket_t.clone());
                return None;
            }
            let index_node = self.stack.pop().unwrap_or(Node::new_nop());
            index_vec.push(index_node);
            // ]
            if !self.cur.eq_kind(TokenKind::BracketR) {
                let msg = format!("変数『{}』への配列アクセスでインデックスの閉じ角括弧がありません。", word_t.label);
                self.throw_error_token(&msg, bracket_t.clone());
                // 書き忘れがあったとして続きを読み進める
            } else {
                self.cur.next(); // "]"
            }
            // n次元配列への代入?
            if self.cur.eq_kind(TokenKind::BracketL) {
                self.cur.next(); // "["
                continue;
            }
            break;
        }
        // "="
        if !self.cur.eq_kind(TokenKind::Eq) { // "="
            // 配列アクセスだけど代入文ではなかった！！
            self.cur.index = old_index; // 巻き戻す
            return None;
        }
        self.cur.next(); // "="
        // value
        let value_node_b = self.check_value();
        if !value_node_b {
            let msg = format!("配列変数『{}』への代入で値が読めません。", word_t.label);
            self.throw_error_token(&msg, word_t);
            return None;
        }
        let value_node = self.stack.pop().unwrap_or(Node::new_nop());
        let var_info = match self.context.find_var_info(&word_t.label) {
            Some(info) => info,
            None => {
                // 配列変数が存在しないのでエラーにする
                let msg = format!("配列変数『{}』への代入がありますが、変数が存在しません。", word_t.label);
                self.throw_error_token(&msg, word_t);
                return None;
            }
        };
        let node_params = NodeValueParamLet{
            var_info,
            value_node: vec![value_node],
            index_node: index_vec,
        };
        let let_array_node = Node::new(NodeKind::ArrayLet, NodeValue::LetVar(node_params), None, word_t.line, self.fileno);
        Some(let_array_node)
    }

    fn check_let(&mut self) -> Option<Node> {
        // ローカル変数の宣言 '変数' がある?
        if self.cur.eq_kind(TokenKind::DefVar) {
            let dainyu = self.cur.peek();
            self.cur.next();
            if self.cur.peek_kind() != TokenKind::Word {
                self.throw_error(format!("『変数の(変数名)』の書式で変数を宣言してください。"), dainyu.line);
                return None;
            }
            // only define local variables
            let word: Token = self.cur.peek();
            self.context.scopes.set_value_local_scope(&word.label, NodeValue::Empty);
            if !self.cur.eq_kinds(&[TokenKind::Word, TokenKind::Eq]) {
                self.cur.next();
                return None; 
            }
        }
        // 配列への代入文か?
        match self.check_let_array() {
            Some(node) => return Some(node),
            None => {}
        }
        // 代入文か?
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
        // -------------------
        // todo: 配列の代入
        // -------------------
        // ローカルに変数があるか？
        let var_name = &word.label;
        let mut var_info:NodeVarInfo = match self.context.find_var_info(&var_name) {
            Some(info) => info,
            None => {
                // グローバル変数を生成
                self.context.scopes.set_value(1, &var_name, NodeValue::Empty);
                let info = self.context.find_var_info(&var_name);
                info.unwrap()
            },
        };
        // 値を得る
        var_info.name = var_name.clone();
        // println!("let:{:?}", var_info);
        let node_value_let = NodeValueParamLet {
            var_info,
            value_node: vec![value],
            index_node: vec![],
        };
        let let_node = Node::new(
            NodeKind::LetVarGlobal, 
            NodeValue::LetVar(node_value_let),
            None, word.line, self.fileno);
        Some(let_node)
    }

    fn check_dainyu(&mut self) -> Option<Node> {
        // VALUE{a}をVAR{b}(に|へ)代入
        let dainyu = self.cur.peek();
        self.cur.next();
        let value_node: Node;
        let var_node: Node;
        let b = self.stack.pop().unwrap_or(Node::new_nop());
        let a = self.stack.pop().unwrap_or(Node::new_nop());
        if b.eq_josi("に") || b.eq_josi("へ") {
            var_node = b;
            value_node = a;
        } else {
            var_node = a;
            value_node = b;
        }
        // get variable name
        let var_name = if var_node.kind == NodeKind::GetVarGlobal {
            match var_node.value {
                NodeValue::GetVar(v) => { v.name },
                _ => { "それ".to_string() }
            }
        } else { String::from("それ") };
        // println!("{}に{:?}を代入", var_name, value_node);
        let mut var_info:NodeVarInfo = match self.context.find_var_info(&var_name) {
            Some(v) => v,
            None => {
                self.context.scopes.set_value(1, &var_name, NodeValue::Empty);
                NodeVarInfo{level:1, no:0, name: String::from("それ")}
            }
        };
        var_info.name = var_name;
        let node_value_let = NodeValueParamLet{var_info, value_node: vec![value_node], index_node: vec![]};
        let let_node = Node::new(
            NodeKind::LetVarGlobal, 
            NodeValue::LetVar(node_value_let), 
            None, dainyu.line, self.fileno);
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
        // カッコの内側の値を読む
        let value_node = self.stack.pop().unwrap_or(Node::new_nop());
        // 続いて閉じ括弧がなければエラー
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
        // JSON?
        if self.cur.eq_kind(TokenKind::BracketL) {
            let t = self.cur.next();
            let mut nlist = vec![];
            loop {
                if self.cur.eq_kind(TokenKind::BracketR) {
                    self.cur.next();
                    break;
                }
                let b = self.check_value_one();
                if b {
                    nlist.push(self.stack.pop().unwrap_or(Node::new_nop()));
                } else {
                    let err_msg = format!("配列データの初期化でエラー。");
                    self.context.throw_error(
                        NodeErrorKind::ParserError, NodeErrorLevel::Error,
                        err_msg, t.line, self.fileno);
                    break;
                }
                if self.cur.eq_kind(TokenKind::Comma) {
                    self.cur.next();
                }
            }
            let nv = NodeValue::NodeList(nlist);
            let ca = Node::new(NodeKind::ArrayCreate, nv, None, t.line, self.fileno);
            self.stack.push(ca);
            return true;
        }
        false
    }

    fn read_func_args(&mut self, func_name: &str, args: Vec<SysArg>, line: u32) -> Vec<Node> {
        let mut arg_nodes = vec![];
        let mut err_msg = String::new();
        let mut sore_hokan = false;
        // todo: 助詞を確認する
        // 引数の数だけstackからpopする
        for _arg in args.iter() {
            let n = match self.stack.pop() {
                Some(n) => n,
                None => {
                    if !sore_hokan {
                        let sore_var = self.context.find_var_info("それ").unwrap_or(NodeVarInfo{level:1, no:0, name:String::from("それ")});
                        let sore_node = Node::new(NodeKind::GetVarGlobal,
                            NodeValue::GetVar(sore_var), None, line, self.fileno);
                        arg_nodes.push(sore_node);
                        sore_hokan = true;
                        continue;
                    }
                    err_msg = format!("{}『{}』の引数が不足しています。", err_msg, String::from(func_name));
                    Node::new_nop()
                }
            };
            arg_nodes.push(n);
        }
        arg_nodes.reverse();
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
        // 変数か関数か？
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
                    NodeValue::CallFunc(name, no, _) => {
                        match meta.kind {
                            NodeVarKind::SysFunc(args) => {
                                let nodes = self.read_func_args(&name, args, word_t.line);
                                let sys_func_node = Node::new(
                                    NodeKind::CallSysFunc, NodeValue::CallFunc(name, no, nodes), 
                                    word_t.josi, word_t.line, self.fileno);
                                sys_func_node
                            },
                            NodeVarKind::UserFunc(args) => {
                                let nodes = self.read_func_args(&name, args, word_t.line);
                                let user_func_node = Node::new(
                                    NodeKind::CallUserFunc, NodeValue::CallFunc(name, no, nodes), 
                                    word_t.josi, word_t.line, self.fileno);
                                    user_func_node
                            },
                            _ => return false,
                        }
                    },
                    // 変数の参照
                    _ => {
                        info.name = String::from(name);
                        let var_node = Node::new(
                            NodeKind::GetVarGlobal,
                            NodeValue::GetVar(info),
                            word_t.josi, word_t.line, self.fileno);
                        var_node
                    }
                }
            },
            // 絶対ある
            None => { return false },
        };
        // 添字があるか？
        if self.cur.peek_kind() == TokenKind::BracketL {
            let mut index_vec = vec![node];
            let t = self.cur.next();
            loop {
                let b = self.check_value();
                if !b {
                    self.throw_error_token(&format!("変数『{}』の配列アクセスで要素が読めません。", name), t);
                    return false;
                }
                if self.cur.peek_kind() != TokenKind::BracketR {
                    self.throw_error_token(&format!("変数『{}』の配列アクセスで閉じ各カッコがありません。", name), t);
                    return false;
                }
                let index_node = self.stack.pop().unwrap_or(Node::new_nop());
                index_vec.push(index_node);
                // n次元配列か？
                self.cur.next(); // skip ']'
                if self.cur.peek_kind() == TokenKind::BracketL {
                    self.cur.next();
                    continue;
                }
                break;
            }
            let ref_node = Node::new(NodeKind::ArrayRef, NodeValue::NodeList(index_vec), None, t.line, self.fileno);
            self.stack.push(ref_node);
        } else {
            self.stack.push(node);
        }
        self.check_operator();
        return true;
    }

    fn check_operator(&mut self) -> bool {
        if !self.cur.eq_operator() { return false; }
        let op_t = self.cur.next();
        let cur_flag = op_t.as_char();
        if !self.check_value_one() {
            self.throw_error_token(&format!("演算子『{}』の後に値がありません。", op_t.label), op_t);
            return false;
        }
        // a [+] (b + c)
        let mut value_bc = self.stack.pop().unwrap_or(Node::new_nop());
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
            match &mut value_bc.value {
                NodeValue::Operator(op) => {
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
                NodeValue::CallFunc(name, no, nodes) => {
                    let value_b = nodes.remove(0);
                    let op_node = Node::new_operator(
                        cur_flag,
                        value_a,
                        value_b,
                        c_josi,
                        op_t.line, self.fileno);
                    nodes.insert(0, op_node);
                    value_bc.value = NodeValue::CallFunc(name.clone(), *no, nodes.clone());
                    self.stack.push(value_bc);
                },
                _ => { self.throw_error_token("計算式のエラー(演算子の入れ替えに失敗)", op_t); return false; }
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
        // 関数を登録 (関数はグローバル領域に確保)
        let scope = &mut self.context.scopes.scopes[1];
        // 変数に名前を登録 - 関数名をスコープに登録
        let no = scope.set_var(&name_t.label, NodeValue::Empty);
        // 関数番号をスコープに再登録(再帰呼び出しに対応)
        scope.set_var(&name_t.label, NodeValue::CallFunc(name_s.clone(), no, vec![]));
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
        let mut body_nodes = match self.get_sentence_list() {
            Ok(nodes) => nodes,
            Err(err) => {
                self.throw_error_token(&format!("関数『{}』の定義でエラー。{}", name_t.label, err), def_t);
                return None;
            },               
        };
        // 「それで戻る」を最後に足す ← TODO: うまく「それ」が追加されていない
        let sore_var = self.context.find_var_info("それ").unwrap_or(NodeVarInfo{level:2, no:0, name:String::from("それ")});
        let sore_node = Node::new(NodeKind::GetVarGlobal,
            NodeValue::GetVar(sore_var), None, name_t.line, self.fileno);
        let ret_node = Node::new(NodeKind::Return, NodeValue::NodeList(vec![sore_node]), None, name_t.line, self.fileno);
        body_nodes.push(ret_node);
        // BlockEnd判定
        if self.cur.eq_kind(TokenKind::BlockEnd) {
            self.cur.next(); // skip ここまで
        }
        // ローカルスコープから抜ける
        self.context.scopes.pop_local();
        // 関数本体を変数に登録
        let func_value: NodeValue = NodeValue::CallFunc(name_s.clone(), no, body_nodes);
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
        assert_eq!(node.kind, NodeKind::LetVarGlobal);
        let let_value = match &node.value {
            NodeValue::LetVar(v) => {
                let name = v.var_info.clone();
                assert_eq!(name.name, "aaa".to_string());
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
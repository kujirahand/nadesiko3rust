//! 構文解析後のノードを定義
use std::collections::HashMap;

/// ノードの種類
#[allow(dead_code)]
#[derive(Debug,PartialEq,Clone,Copy)]
pub enum NodeKind {
    Nop,
    Comment,
    NodeList,
    Int,
    Bool,
    Number,
    String,
    GetVarGlobal, // グローバル変数の取得
    LetVarGlobal, // グローバル変数への代入
    GetVarLocal,
    LetVarLocal,
    Operator,
    CallSysFunc,
    CallUserFunc,
    If,
    Kai,
    Break,
    Continue,
    Return,
    For,
    ArrayCreate,
    ArrayRef,
    ArrayLet,
}

/// ノード構造体
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub value: NodeValue,
    pub josi: Option<String>,
    pub line: u32,
    pub fileno: u32,
}
impl Node {
    /// ノードを文字列変換する
    pub fn to_string(&self) -> String {
        match self.kind {
            NodeKind::Nop => String::from("Nop"),
            NodeKind::NodeList => format!("NodeList:{}", self.value.to_string()),
            NodeKind::Int => format!("I({})", self.value.to_int(0)),
            NodeKind::Number => format!("F({})", self.value.to_float(0.0)),
            NodeKind::Bool => format!("B({})", self.value.to_string()),
            NodeKind::String => format!("\"{}\"", self.value.to_string()),
            NodeKind::Comment => format!("Comment:{}", self.value.to_string()),
            NodeKind::LetVarGlobal => format!("Let:{}", self.value.to_string()),
            NodeKind::GetVarGlobal => format!("Get:{}", self.value.to_string()),
            NodeKind::LetVarLocal => format!("LetVarLocal:{}", self.value.to_string()),
            NodeKind::GetVarLocal => format!("GetVarLocal:{}", self.value.to_string()),
            NodeKind::Operator => format!("{}", self.value.to_string()),
            NodeKind::CallSysFunc => format!("Sys:{}", self.value.to_string()),
            NodeKind::CallUserFunc => format!("Usr:{}", self.value.to_string()),
            NodeKind::If => format!("If:{}", self.value.to_string()),
            NodeKind::Kai => format!("N回:{}", self.value.to_string()),
            NodeKind::Break => String::from("Break"),
            NodeKind::Continue => String::from("Continue"),
            NodeKind::For => String::from("For"),
            NodeKind::Return => format!("戻る:{}", self.value.to_string()),
            NodeKind::ArrayCreate => format!("配列生成"),
            NodeKind::ArrayRef => format!("配列参照"),
            NodeKind::ArrayLet => format!("配列代入"),
            // _ => format!("{:?}", self.kind),
        }
    }
    /// 新規ノードを作成
    pub fn new(kind: NodeKind, value: NodeValue, josi: Option<String>, line: u32, fileno: u32) -> Self {
        Self {
            kind,
            value,
            josi,
            line, 
            fileno
        }
    }
    pub fn new_nop() -> Self {
        Node::new(NodeKind::Nop, NodeValue::Empty, None, 0, 0)
    }
    pub fn new_operator(operator: char, node_l: Node, node_r: Node, josi: Option<String>, line: u32, fileno: u32) -> Self {
        Node::new(
            NodeKind::Operator, 
            NodeValue::Operator(NodeValueParamOperator {
                flag: operator,
                nodes: vec![node_l, node_r],
            }),
            josi, line, fileno
        )
    }
    pub fn new_node_list(list: Vec<Node>, line: u32, fileno: u32) -> Self {
        Node::new(
            NodeKind::NodeList,
            NodeValue::NodeList(list),
            None,
            line,
            fileno
        )
    }
    pub fn get_josi_str(&self) -> String {
        match &self.josi {
            Some(josi_str) =>  josi_str.clone(),
            None => String::from(""),
        }
    }
    pub fn eq_josi(&self, dest_josi: &str) -> bool {
        match &self.josi {
            Some(j) => j.eq(dest_josi),
            None => dest_josi == "",
        }
    }
    pub fn is_renbun_josi(&self) -> bool {
        match &self.josi {
            None => false,
            Some(j) => j.eq("して") || j.eq("って") || j.eq("きて"),
        }
    }
}

// I to B => (i != FALSE_VALUE)
const FALSE_VALUE:isize = 0;
const TRUE_VALUE:isize = 1;

/// ノードの値を定義したもの
#[derive(Debug,Clone)]
pub enum NodeValue {
    Empty,
    S(String),
    I(isize),
    F(f64),
    B(bool),
    A(Vec<NodeValue>),
    NodeList(Vec<Node>),
    LetVar(NodeValueParamLet),
    GetVar(NodeVarInfo),
    Operator(NodeValueParamOperator),
    CallFunc(String, usize, Vec<Node>), // 関数(FuncNo, Args) CallFuncNo link to context.CallFuncs[FuncNo]
}
impl NodeValue {
    pub fn from_str(v: &str) -> Self {
        Self::S(v.to_string())
    }
    pub fn to_string(&self) -> String {
        match self {
            NodeValue::Empty => String::from(""),
            NodeValue::S(v) => format!("{}", v),
            NodeValue::I(v) => format!("{}", v),
            NodeValue::F(v) => format!("{}", v),
            NodeValue::B(v) => if *v { String::from("真") } else { String::from("偽") },
            NodeValue::A(v) => format!("A[len({})]", v.len()),
            NodeValue::LetVar(v) => format!("{}={}", v.var_info.name, nodes_to_string(&v.value_node, ",")),
            NodeValue::NodeList(nodes) => format!("[{}]", nodes_to_string(&nodes, ",")),
            NodeValue::Operator(op) => format!("({})[{}]", op.flag, nodes_to_string(&op.nodes, ",")),
            NodeValue::GetVar(v) => format!("{}", v.name.clone()),
            NodeValue::CallFunc(name, _no, nodes) => format!("{}({})", name, nodes_to_string(&nodes, ",")),
            // _ => String::from(""),
        }
    }
    pub fn to_bool(&self) -> bool {
        match self {
            NodeValue::B(v) => *v,
            _ => {
                let v = self.to_int(0);
                v != FALSE_VALUE
            }
        }
    }
    pub fn to_int(&self, def_value: isize) -> isize {
        match self {
            NodeValue::Empty => def_value,
            NodeValue::S(v) => v.parse().unwrap_or(def_value),
            NodeValue::I(v) => *v,
            NodeValue::F(v) => *v as isize,
            NodeValue::CallFunc(_, v, _) => *v as isize,
            NodeValue::B(v) => if *v { TRUE_VALUE } else { FALSE_VALUE },
            _ => def_value,
        }
    }
    pub fn to_float(&self, def_value: f64) -> f64 {
        match self {
            NodeValue::Empty => def_value,
            NodeValue::S(v) => v.parse().unwrap_or(def_value),
            NodeValue::I(v) => *v as f64,
            NodeValue::F(v) => *v as f64,
            NodeValue::B(v) => if *v { TRUE_VALUE as f64 } else { FALSE_VALUE as f64 }
            _ => def_value,
        }
    }
    pub fn to_nodes(&self) -> Vec<Node> {
        match self {
            NodeValue::NodeList(nodes) => return nodes.clone(),
            _ => vec![],
        }
    }
    pub fn get_array_index(&self, index: usize) -> Option<NodeValue> {
        return match self {
            NodeValue::A(nlist) => {
                let v = nlist[index].clone();
                return Some(v);
            }
            _ => None
        }
    }
    pub fn get_array_index_mut(&mut self, index: usize) -> Option<&mut NodeValue> {
        return match self {
            NodeValue::A(nlist) => {
                return nlist.get_mut(index);
            }
            _ => None
        }
    }
    pub fn set_array_index(&mut self, index: usize, value: NodeValue) -> bool {
        match self {
            NodeValue::A(nlist) => {
                while nlist.len() <= index {
                    nlist.push(NodeValue::Empty);
                }
                nlist[index] = value;
                return true;
            }
            _ => {},
        }
        return false;
    }
}

/// NodeValue 同士の計算を行うメソッドを定義
impl NodeValue {
    pub fn calc_plus(left: &NodeValue, right: &NodeValue) -> NodeValue {
        match (left, right) {
            // number
            (NodeValue::I(lv), NodeValue::I(rv)) => NodeValue::I(lv + rv),
            (NodeValue::F(lv), NodeValue::I(rv)) => NodeValue::F(lv + *rv as f64),
            (NodeValue::I(lv), NodeValue::F(rv)) => NodeValue::F(*lv as f64 + rv),
            (NodeValue::F(lv), NodeValue::F(rv)) => NodeValue::F(lv + rv),
            // string
            (NodeValue::S(lv), NodeValue::S(rv)) => NodeValue::S(format!("{}{}", lv, rv)),
            // string + number
            (NodeValue::S(lv), NodeValue::I(rv)) => NodeValue::I(lv.parse().unwrap_or(0) as isize + rv),
            (NodeValue::S(lv), NodeValue::F(rv)) => NodeValue::F(lv.parse().unwrap_or(0.0) as f64 + rv),
            // other
            _ => NodeValue::Empty,
        }
    }
    pub fn calc_plus_str(left: &NodeValue, right: &NodeValue) -> NodeValue {
        let s = format!("{}{}", left.to_string(), right.to_string());
        NodeValue::S(s)
    }
    pub fn calc_minus(left: &NodeValue, right: &NodeValue) -> NodeValue {
        match right {
            NodeValue::I(rv) => NodeValue::I(left.to_int(0) - rv),
            NodeValue::F(rv) => NodeValue::F(left.to_float(0.0) - rv),
            _ => NodeValue::Empty,
        }
    }
    pub fn calc_mul(left: &NodeValue, right: &NodeValue) -> NodeValue {
        match (left, right) {
            (NodeValue::I(lv), NodeValue::I(rv)) => NodeValue::I(lv * rv),
            (NodeValue::I(lv), _) => NodeValue::F((*lv as f64) * right.to_float(0.0)),
            (NodeValue::F(lv), _) => NodeValue::F(*lv * right.to_float(0.0)),
            (NodeValue::S(lv), NodeValue::I(times)) => NodeValue::S(Self::repeat_str(lv, *times as usize)),
            (_, _) => NodeValue::Empty,
        }
    }
    fn repeat_str(s: &str, times: usize) -> String {
        let mut res = String::new();
        for _ in 0..times {
            res.push_str(s);
        }
        res
    }
    pub fn calc_div(left: &NodeValue, right: &NodeValue) -> NodeValue {
        match (left, right) {
            (NodeValue::I(lv), NodeValue::I(rv)) => NodeValue::F(*lv as f64 / *rv as f64),
            (NodeValue::I(lv), NodeValue::F(rv)) => NodeValue::F((*lv as f64) / *rv as f64),
            (NodeValue::F(lv), NodeValue::I(rv)) => NodeValue::F((*lv as f64) / *rv as f64),
            (NodeValue::F(lv), NodeValue::F(rv)) => NodeValue::F((*lv as f64) / *rv as f64),
            (NodeValue::S(_), _) => NodeValue::F(left.to_float(0.0) / right.to_float(0.0)),
            (_, _) => NodeValue::Empty,
        }
    }
    pub fn calc_mod(left: &NodeValue, right: &NodeValue) -> NodeValue {
        match (left, right) {
            (NodeValue::I(lv), NodeValue::I(rv)) => NodeValue::I(*lv % *rv),
            (NodeValue::I(lv), NodeValue::F(rv)) => NodeValue::F((*lv as f64) % *rv as f64),
            (NodeValue::F(lv), NodeValue::I(rv)) => NodeValue::F((*lv as f64) % *rv as f64),
            (NodeValue::F(lv), NodeValue::F(rv)) => NodeValue::F((*lv as f64) % *rv as f64),
            (NodeValue::S(_), _) => NodeValue::F(left.to_float(0.0) / right.to_float(0.0)),
            (_, _) => NodeValue::Empty,
        }
    }
    pub fn calc_eq(left: &NodeValue, right: &NodeValue) -> NodeValue {
        NodeValue::B(left.to_int(0) == right.to_int(0))
    }
    pub fn calc_noteq(left: &NodeValue, right: &NodeValue) -> NodeValue {
        NodeValue::B(left.to_int(0) != right.to_int(0))
    }
    pub fn calc_gt(left: &NodeValue, right: &NodeValue) -> NodeValue {
        NodeValue::B(left.to_float(0.0) > right.to_float(0.0))
    }
    pub fn calc_gteq(left: &NodeValue, right: &NodeValue) -> NodeValue {
        NodeValue::B(left.to_float(0.0) >= right.to_float(0.0))
    }
    pub fn calc_lt(left: &NodeValue, right: &NodeValue) -> NodeValue {
        NodeValue::B(left.to_float(0.0) < right.to_float(0.0))
    }
    pub fn calc_lteq(left: &NodeValue, right: &NodeValue) -> NodeValue {
        NodeValue::B(left.to_float(0.0) <= right.to_float(0.0))
    }
    pub fn calc_and(left: &NodeValue, right: &NodeValue) -> NodeValue {
        NodeValue::B(left.to_bool() && right.to_bool())
    }
    pub fn calc_or(left: &NodeValue, right: &NodeValue) -> NodeValue {
        NodeValue::B(left.to_bool() || right.to_bool())
    }
    pub fn calc_pow(left: &NodeValue, right: &NodeValue) -> NodeValue {
        match (left, right) {
            (NodeValue::I(l),NodeValue::I(r)) => NodeValue::I((*l).pow(*r as u32)),
            (NodeValue::F(l),NodeValue::I(r)) => NodeValue::F((*l).powi(*r as i32)),
            (NodeValue::F(l),NodeValue::F(r)) => NodeValue::F((*l).powf(*r)),
            (_, _) => NodeValue::Empty,
        }
    }
}

#[derive(Debug,Clone)]
pub struct NodeValueParamLet {
    pub var_info: NodeVarInfo,
    pub value_node: Vec<Node>,
    pub index_node: Vec<Node>,
}

#[derive(Debug,Clone)]
pub struct NodeVarInfo {
    pub level: usize,
    pub no: usize,
    pub name: String,
}

#[derive(Debug,Clone,PartialEq)]
pub struct NodeVarMeta {
    pub read_only: bool,
    pub kind: NodeVarKind,
}
impl NodeVarMeta {
    pub fn new() -> Self {
        Self {
            read_only: false,
            kind: NodeVarKind::Empty,
        }
    }
}

#[derive(Debug,Clone)]
pub struct NodeScopeList {
    pub scopes: Vec<NodeScope>,
}
impl NodeScopeList {
    pub fn new() -> Self {
        // generate system and global
        let sys_scope = NodeScope::new();
        let user_global = NodeScope::new();
        // 0: SYSTEM / 1: USER_GLOBAL / 2: LOCAL ...
        let scopes = vec![sys_scope, user_global];
        Self { scopes }
    }
    pub fn len(&self) -> usize {
        self.scopes.len()
    }
    pub fn push_local(&mut self, scope: NodeScope) -> usize {
        self.scopes.push(scope);
        self.scopes.len()
    }
    pub fn pop_local(&mut self) -> Option<NodeScope> {
        if self.scopes.len() >= 3 {
            return self.scopes.pop();
        }
        None
    }
    pub fn find_var(&self, name: &str) -> Option<NodeVarInfo> {
        let mut i: isize = (self.scopes.len() - 1) as isize;
        while i >= 0 {
            let scope = &self.scopes[i as usize];
            if let Some(no) = scope.find_var(name) {
                return Some(NodeVarInfo {
                    level: i as usize,
                    no: *no,
                    name: String::from(name),
                })
            }
            i -= 1;
        }
        None
    }
    pub fn set_value(&mut self, level: usize, name: &str, value: NodeValue) -> usize {
        // local or global
        let level = if level >= 2 {
            while self.scopes.len() <= 2 {
                self.scopes.push(NodeScope::new());
            }
            self.scopes.len() - 1
        } else { level };
        let scope = &mut self.scopes[level];
        scope.set_var(name, value)
    }
    pub fn set_value_local_scope(&mut self, name: &str, value: NodeValue) -> NodeVarInfo {
        let local = self.scopes.len() - 1;
        let no = self.set_value(local, name, value);
        NodeVarInfo {
            name: String::from(name),
            level: local,
            no
        }
    }
    pub fn get_var_value(&self, info: &NodeVarInfo) -> Option<NodeValue> {
        let scope: &NodeScope = &self.scopes[info.level];
        if scope.var_values.len() > info.no {
            return Some(scope.var_values[info.no].clone());
        }
        None
    }
    pub fn get_var_value_mut(&mut self, info: &NodeVarInfo) -> Option<&mut NodeValue> {
        let scope: &mut NodeScope = &mut self.scopes[info.level];
        if scope.var_values.len() > info.no {
            return scope.var_values.get_mut(info.no);
        }
        return None;
    }
    pub fn get_var_meta(&self, info: &NodeVarInfo) -> Option<NodeVarMeta> {
        let scope: &NodeScope = &self.scopes[info.level];
        if scope.var_values.len() > info.no {
            return Some(scope.var_metas[info.no].clone());
        }
        None
    }
}


#[derive(Debug,Clone)]
pub struct NodeScope {
    pub var_names: HashMap<String, usize>,
    pub var_values: Vec<NodeValue>,
    pub var_metas: Vec<NodeVarMeta>,
}
impl NodeScope {
    pub fn new() -> Self {
        // prepare
        let var_names = HashMap::new();
        let var_values = vec![];
        let var_metas = vec![];
        let mut obj = Self {
            var_names,
            var_values,
            var_metas,
        };
        // add sore
        obj.set_var("それ", NodeValue::Empty);
        obj
    }

    pub fn get_var(&self, name: &str) -> NodeValue {
        let no = match self.find_var(name) {
            Some(no) => *no,
            None => return NodeValue::Empty,
        };
        self.var_values[no].clone()
    }

    pub fn find_var(&self, name: &str) -> Option<&usize> {
        self.var_names.get(name)
    }

    pub fn set_var(&mut self, name: &str, new_value: NodeValue) -> usize {
        match self.var_names.get(name) {
            None => {
                let no = self.var_values.len();
                self.var_values.push(new_value);
                self.var_metas.push(NodeVarMeta::new());
                self.var_names.insert(String::from(name), no);
                no
            },
            Some(no) => {
                self.var_values[*no] = new_value;
                *no
            }
        }
    }
}

#[derive(Debug,Clone)]
pub struct NodeValueParamOperator {
    pub flag: char,
    pub nodes: Vec<Node>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NodeVarKind {
    Empty,
    Bool,
    Number,
    String,
    SysFunc(Vec<SysArg>),
    UserFunc(Vec<SysArg>),
    Array,
    Dict,
}

pub fn nodes_to_string(nodes: &Vec<Node>, delimiter: &str) -> String {
    let mut r = String::new();
    for (i, node) in nodes.iter().enumerate() {
        if delimiter.eq("\n") {
            let line = format!("L{}: {}", &node.line, &node.to_string());
            r.push_str(&line);
        } else {
            let line = format!("{}", &node.to_string());
            r.push_str(&line);
        }
        if i != (nodes.len() - 1) {
            r.push_str(delimiter);
        }
    }
    r
}

#[derive(Clone)]
pub struct NodeContext {
    pub debug_mode: bool,
    pub callstack_level: usize,
    pub labels: HashMap<String, Node>,
    pub scopes: NodeScopeList,
    pub files: Vec<String>,
    pub sysfuncs: Vec<SysFuncInfo>,
    errors: Vec<NodeError>,
    error_count: usize,
    pub try_break: Option<usize>,
    pub try_continue: Option<usize>,
    pub try_return: Option<usize>,
    pub return_level: usize,
    pub print_log: String,
}

impl NodeContext {
    pub fn new() -> Self {
        NodeContext {
            debug_mode: false,
            callstack_level: 0,
            labels: HashMap::new(),
            scopes: NodeScopeList::new(),
            files: vec![],
            sysfuncs: vec![],
            errors: vec![],
            error_count: 0,
            try_break: None,
            try_continue: None,
            try_return: None,
            return_level: 0,
            print_log: String::new(),
        }
    }
    // for file management
    pub fn set_filename(&mut self, filename: &str) -> u32 {
        match self.find_files(filename) {
            Some(fileno) => fileno,
            None => {
                let fileno = self.files.len() as u32;
                self.files.push(filename.to_string());
                fileno
            },
        }
    }
    pub fn find_files(&self, filename: &str) -> Option<u32> {
        for (i, fname) in self.files.iter().enumerate() {
            if fname == filename { return Some(i as u32); }
        }
        None
    }
    // for error
    pub fn has_error(&self) -> bool {
        self.error_count > 0
    }
    pub fn get_error_str(&self) -> String {
        let mut res = String::new();
        for e in self.errors.iter() {
            res.push_str(&format!("{}\n", e.to_string()));
        }
        res
    }
    pub fn throw_error(&mut self, kind: NodeErrorKind, level: NodeErrorLevel, msg: String, line: u32, fileno: u32) {
        let err = NodeError::new(kind, level, msg, line, fileno);
        println!("{}", &err.to_string());
        self.errors.push(err);
        match level {
            NodeErrorLevel::Error => self.error_count += 1,
            _ => {},
        }
    }
    pub fn throw_runtime_error(&mut self, msg: String, line: u32, fileno: u32) {
        self.throw_error(NodeErrorKind::RuntimeError, NodeErrorLevel::Error, msg, line, fileno);
    }
    // for scope variables
    pub fn find_var_info(&self, name: &str) -> Option<NodeVarInfo> {
        self.scopes.find_var(name)
    }
    pub fn get_var_value(&self, info: &NodeVarInfo) -> Option<NodeValue> {
        if info.level >= 2 {
            // local
            let local: &NodeScope = self.scopes.scopes.last().unwrap();
            if local.var_values.len() > info.no {
                return Some(local.var_values[info.no].clone())
            } else {
                return None;
            }
        } else {
            // system or global 
            self.scopes.get_var_value(info)
        }
    }
    #[allow(dead_code)]
    pub fn get_var_meta(&self, info: &NodeVarInfo) -> Option<NodeVarMeta> {
        self.scopes.get_var_meta(info)
    }
    // add system func
    pub fn add_sysfunc(&mut self, name: &str, args: Vec<SysArg>, func: SysFuncType) -> usize {
        // add func to sysfuncs
        let sys_no = self.sysfuncs.len();
        let sfi = SysFuncInfo{ func };
        self.sysfuncs.push(sfi);
        // add name to scope
        let scope = &mut self.scopes.scopes[0];
        let no = scope.set_var(name, NodeValue::CallFunc(String::from(name), sys_no, vec![]));
        scope.var_metas[no].read_only = true;
        scope.var_metas[no].kind = NodeVarKind::SysFunc(args);
        sys_no     
    }
    pub fn add_sysvar(&mut self, name: &str, value: NodeValue) -> usize {
        let no = self.add_sysconst(name, value);
        let scope = &mut self.scopes.scopes[0];
        scope.var_metas[no].read_only = false;
        no
    }
    pub fn add_sysconst(&mut self, name: &str, value: NodeValue) -> usize {
        let kind =  match &value {
            NodeValue::B(_) => { NodeVarKind::Bool },
            NodeValue::S(_) => { NodeVarKind::String },
            NodeValue::I(_) => { NodeVarKind::Number },
            NodeValue::F(_) => { NodeVarKind::Number },
            _ => { NodeVarKind::Empty }
        };
        let scope = &mut self.scopes.scopes[0];
        let no = scope.set_var(name, value);
        scope.var_metas[no].read_only = true;
        scope.var_metas[no].kind = kind;
        no
    }
}

type SysFuncType = fn(&mut NodeContext, Vec<NodeValue>) -> Option<NodeValue>;

#[derive(Clone)]
pub struct SysFuncInfo {
    pub func: SysFuncType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SysArg {
    pub name: String, 
    pub josi_list: Vec<String>
}

pub fn sysargs(args: &[&[&str]]) -> Vec<SysArg> {
    let mut result = vec![];
    let mut a: char = 'A';
    for arg in args.iter() {
        let mut arg_res: Vec<String> = vec![];
        for a in arg.iter() {
            arg_res.push(String::from(*a));
        }
        let a_name = String::from(a);
        result.push(SysArg{ name: a_name, josi_list: arg_res });
        a = ((a as u8) + 1u8) as char;
    }
    result
}

#[allow(dead_code)]
#[derive(Debug,Clone,Copy)]
pub enum NodeErrorKind {
    ParserError,
    RuntimeError,
}
#[allow(dead_code)]
#[derive(Debug,Clone,Copy)]
pub enum NodeErrorLevel {
    Hint, Warning, Error
}

#[derive(Debug,Clone)]
pub struct NodeError {
    pub kind: NodeErrorKind,
    pub level: NodeErrorLevel,
    pub message: String,
    pub line: u32,
    pub fileno: u32,
}

impl NodeError {
    pub fn new(kind: NodeErrorKind, level: NodeErrorLevel, message: String, line: u32, fileno: u32) -> NodeError {
        Self {
            kind,
            level,
            message,
            line,
            fileno
        }
    }
    pub fn to_string(&self) -> String {
        let kind_str = match self.kind {
            NodeErrorKind::ParserError => "解析時",
            NodeErrorKind::RuntimeError => "実行時"
        };
        let level_str = match self.level {
            NodeErrorLevel::Error => "エラー",
            NodeErrorLevel::Warning => "警告",
            NodeErrorLevel::Hint => "ヒント",
        };
        format!("[{}{}]({}){}", kind_str, level_str, self.line, self.message)
    } 
}


use std::collections::HashMap;

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
    GetVar, // グローバル変数の取得
    Let, // グローバル変数への代入
    Operator,
    CallSysFunc,
    If,
    Kai,
    Break,
    Continue,
    For,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub value: NodeValue,
    pub josi: Option<String>,
    pub line: u32,
    pub fileno: u32,
}
impl Node {
    pub fn to_string(&self) -> String {
        match self.kind {
            NodeKind::NodeList => format!("NodeList:{}", self.value.to_string()),
            NodeKind::Int => format!("Int:{}", self.value.to_int(0)),
            NodeKind::Number => format!("Number:{}", self.value.to_float(0.0)),
            NodeKind::Bool => format!("Bool:{}", self.value.to_string()),
            NodeKind::Comment => format!("Comment:{}", self.value.to_string()),
            NodeKind::Let => format!("Let:{}", self.value.to_string()),
            NodeKind::GetVar => format!("{}", self.value.to_string()),
            NodeKind::Operator => format!("{}", self.value.to_string()),
            NodeKind::String => format!("\"{}\"", self.value.to_string()),
            NodeKind::CallSysFunc => format!("Call:{}", self.value.to_string()),
            NodeKind::If => format!("If:{}", self.value.to_string()),
            NodeKind::Kai => format!("N回:{}", self.value.to_string()),
            NodeKind::Nop => String::from("Nop"),
            NodeKind::Break => String::from("Break"),
            NodeKind::Continue => String::from("Continue"),
            NodeKind::For => String::from("For"),
            // _ => format!("{:?}", self.kind),
        }
    }
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
            NodeValue::Operator(NodeValueOperator {
                flag: operator,
                nodes: vec![node_l, node_r]
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
}

// I to B => (i != FALSE_VALUE)
const FALSE_VALUE:isize = 0;
const TRUE_VALUE:isize = 1;

#[derive(Debug,Clone)]
pub enum NodeValue {
    Empty,
    S(String),
    I(isize),
    F(f64),
    B(bool),
    NodeList(Vec<Node>),
    LetVar(NodeValueLet),
    GetVar(NodeVarInfo),
    Operator(NodeValueOperator),
    SysFunc(String, usize, Vec<Node>), // (FuncNo, Args) SysFuncNo link to context.sysfuncs[FuncNo]
}
impl NodeValue {
    pub fn to_string(&self) -> String {
        match self {
            NodeValue::Empty => String::from("Empty"),
            NodeValue::S(v) => format!("{}", v),
            NodeValue::I(v) => format!("{}", v),
            NodeValue::F(v) => format!("{}", v),
            NodeValue::B(v) => if *v { String::from("真") } else { String::from("偽") },
            NodeValue::LetVar(v) => format!("LetVar{}={:?}", v.var_name, v.value_node),
            NodeValue::NodeList(nodes) => format!("NodeList:[{}]", nodes_to_string(&nodes, ",")),
            NodeValue::Operator(op) => format!("{}[{}]", op.flag, nodes_to_string(&op.nodes, ",")),
            NodeValue::GetVar(v) => format!("GetVar:{:?}({},{})", v.name.clone().unwrap_or(String::new()), v.level, v.no),
            NodeValue::SysFunc(name, _, nodes) => format!("SysFunc:{}({})", name, nodes_to_string(&nodes, ",")),
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
            NodeValue::SysFunc(_, v, _) => *v as isize,
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
}

impl NodeValue {
    // calc method
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
pub struct NodeValueLet {
    pub var_name: String,
    pub value_node: Vec<Node>,
}

#[derive(Debug,Clone)]
pub struct NodeVarInfo {
    pub level: usize,
    pub no: usize,
    pub name: Option<String>,
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
        let scopes = vec![sys_scope, user_global];
        Self { scopes }
    }
    pub fn find_var(&self, name: &str) -> Option<NodeVarInfo> {
        let mut i: isize = (self.scopes.len() - 1) as isize;
        while i >= 0 {
            let scope = &self.scopes[i as usize];
            if let Some(no) = scope.find_var(name) {
                return Some(NodeVarInfo {
                    level: i as usize,
                    no: *no,
                    name: None, // 検索では変数名は返さない
                })
            }
            i -= 1;
        }
        None
    }
    pub fn set_value(&mut self, level: usize, name: &str, value: NodeValue) -> usize {
        while self.scopes.len() <= level {
            self.scopes.push(NodeScope::new());
        }
        let scope = &mut self.scopes[level];
        scope.set_var(name, value)
    }
    pub fn set_value_local_scope(&mut self, name: &str, value: NodeValue) -> NodeVarInfo {
        let local = self.scopes.len() - 1;
        let no = self.set_value(local, name, value);
        NodeVarInfo {
            name: None,
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
pub struct NodeValueOperator {
    pub flag: char,
    pub nodes: Vec<Node>,
}

#[allow(dead_code)]
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum NodeVarKind {
    Empty,
    Number,
    String,
    SysFunction,
    Function,
    Array,
    Dict,
}

pub fn nodes_to_string(nodes: &Vec<Node>, delimiter: &str) -> String {
    let mut r = String::new();
    for (i, node) in nodes.iter().enumerate() {
        r.push_str(&node.to_string());
        if i != (nodes.len() - 1) {
            r.push_str(delimiter);
        }
    }
    r
}


use std::collections::HashMap;

#[derive(Debug,PartialEq,Clone,Copy)]
pub enum NodeKind {
    Nop,
    Comment,
    List,
    Int,
    Number,
    String,
    StringEx,
    GetVar,
    Let,
    DebugPrint,
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
    pub fn new_nop() -> Self {
        Node::new(NodeKind::Nop, NodeValue::Empty, 0, 0)
    }
    pub fn to_string(&self) -> String {
        match self.kind {
            NodeKind::Int => format!("Int:{}", self.value.to_string()),
            NodeKind::Comment => format!("// {}", self.value.to_string()),
            NodeKind::Let => format!("Let:{}", self.value.to_string()),
            _ => format!("{:?}", self.kind),
        }
    }
}

#[derive(Debug,Clone)]
pub enum NodeValue {
    Empty,
    S(String),
    I(isize),
    F(f64),
    Nodes(Vec<Node>),
    LetVar(NodeValueLet),
    GetVar(NodeVarInfo),
}
impl NodeValue {
    pub fn to_string(&self) -> String {
        match self {
            NodeValue::Empty => String::from("Empty"),
            NodeValue::S(v) => format!("{}", v),
            NodeValue::I(v) => format!("{}", v),
            NodeValue::F(v) => format!("{}", v),
            NodeValue::LetVar(v) => format!("{}={:?}", v.var_name, v.value_node),
            NodeValue::Nodes(nodes) => format!("Nodes:{}", nodes_to_string(nodes, ",")),
            _ => String::from(""),
        }
    }
}

#[derive(Debug,Clone)]
pub struct NodeValueLet {
    pub var_name: String,
    pub var_info: NodeVarInfo,
    pub value_node: Vec<Node>,
}

#[derive(Debug,Clone)]
pub struct NodeContext {
    pub index: usize,
    pub callstack_level: usize,
    pub labels: HashMap<String, Node>,
    pub scopes: Vec<NodeScope>,
}

impl NodeContext {
    pub fn new() -> Self {
        let sys_global = NodeScope::new();
        let user_global = NodeScope::new();
        let scopes = vec![sys_global, user_global];
        NodeContext {
            index: 0,
            callstack_level: 0,
            labels: HashMap::new(),
            scopes,
        }
    }
    pub fn find_var_info(&self, name: &str) -> Option<NodeVarInfo> {
        // 末端から変数名を検索
        let mut level: isize = (self.scopes.len() - 1) as isize;
        while level >= 0 {
            let scope = &self.scopes[level as usize];
            match scope.get_var_no(name) {
                Some(no) => return Some(NodeVarInfo{name: None, level: level as usize, no:*no}),
                None => {
                    level -= 1;
                    continue;
                }
            }
        }
        None
    }
    pub fn get_var_value(&self, i: &NodeVarInfo) -> Option<NodeValue> {
        if i.level >= self.scopes.len() {
            return None;
        }
        let scope = &self.scopes[i.level];
        Some(scope.var_values[i.no].clone())
    }
}

#[derive(Debug,Clone)]
pub struct NodeVarInfo {
    pub level: usize,
    pub no: usize,
    pub name: Option<String>,
}

#[derive(Debug,Clone)]
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
    pub fn get_var_no(&self, name: &str) -> Option<&usize> {
        self.var_names.get(name)
    }
    pub fn get_var_value(&self, name: &str) -> Option<NodeValue> {
        match self.var_names.get(name) {
            None => None,
            Some(no) => Some(self.var_values[*no].clone()),
        }
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

#[derive(Debug,Clone,Copy)]
pub enum NodeVarKind {
    Empty,
    Number,
    String,
    Function,
    Array,
    Dict,
}

pub fn nodes_to_string(nodes: &Vec<Node>, delimiter: &str) -> String {
    let mut r = String::new();
    for node in nodes.iter() {
        r.push_str(&format!("{}{}", node.to_string(), delimiter));
    }
    r
}

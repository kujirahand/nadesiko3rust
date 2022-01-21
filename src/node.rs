use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug,PartialEq,Clone,Copy)]
pub enum NodeKind {
    Nop,
    Comment,
    Int,
    Number,
    String,
    StringEx,
    GetVar,
    Let,
    Operator,
    CallSysFunc,
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
    pub fn new_operator(operator: char, node_l: Node, node_r: Node, line: u32, fileno: u32) -> Self {
        Node::new(
            NodeKind::Operator, 
            NodeValue::Operator(NodeValueOperator {
                flag: operator,
                nodes: vec![node_l, node_r]
            }),
            None, line, fileno
        )
    }
    pub fn get_josi_str(&self) -> String {
        match &self.josi {
            Some(josi_str) =>  josi_str.clone(),
            None => String::from(""),
        }
    }
    pub fn to_string(&self) -> String {
        match self.kind {
            NodeKind::Int => format!("Int:{}", self.value.to_string()),
            NodeKind::Comment => format!("Comment:{}", self.value.to_string()),
            NodeKind::Let => format!("Let:{}", self.value.to_string()),
            NodeKind::GetVar => format!("GetVar:{}", self.value.to_string()),
            NodeKind::Operator => format!("Operator:{}", self.value.to_string()),
            NodeKind::CallSysFunc => format!("CallSysFunc:{}", self.value.to_string()),
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
    Operator(NodeValueOperator),
    SysFunc(usize, Vec<Node>), // (FuncNo, Args) SysFuncNo link to context.sysfuncs[FuncNo]
}
impl NodeValue {
    pub fn to_string(&self) -> String {
        match self {
            NodeValue::Empty => String::from("Empty"),
            NodeValue::S(v) => format!("{}", v),
            NodeValue::I(v) => format!("{}", v),
            NodeValue::F(v) => format!("{}", v),
            NodeValue::LetVar(v) => format!("{}={:?}", v.var_name, v.value_node),
            NodeValue::Nodes(nodes) => format!("Nodes:[{}]", nodes_to_string(&nodes, ",")),
            NodeValue::Operator(op) => format!("Operator:{}[{}]", op.flag, nodes_to_string(&op.nodes, ",")),
            NodeValue::GetVar(var) => format!("GetVar:{:?}", var),
            NodeValue::SysFunc(no, nodes) => format!("SysFunc:{}:[{}]", no, nodes_to_string(&nodes, ",")),
            // _ => String::from(""),
        }
    }
    pub fn to_int(&self, def_value: isize) -> isize {
        match self {
            NodeValue::Empty => def_value,
            NodeValue::S(v) => v.parse().unwrap_or(def_value),
            NodeValue::I(v) => *v,
            NodeValue::F(v) => *v as isize,
            NodeValue::SysFunc(v, _) => *v as isize,
            _ => def_value,
        }
    }
    pub fn to_float(&self, def_value: f64) -> f64 {
        match self {
            NodeValue::Empty => def_value,
            NodeValue::S(v) => v.parse().unwrap_or(def_value),
            NodeValue::I(v) => *v as f64,
            NodeValue::F(v) => *v as f64,
            _ => def_value,
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
}

#[derive(Debug,Clone)]
pub struct NodeValueLet {
    pub var_name: String,
    pub var_info: NodeVarInfo,
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


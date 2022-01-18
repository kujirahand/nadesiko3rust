#[derive(Debug,PartialEq,Clone,Copy)]
pub enum NodeKind {
    Nop,
    Comment,
    List,
    Int,
    Number,
    String,
    StringEx,
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
}

#[derive(Debug,Clone)]
pub enum NodeValue {
    Empty,
    S(String),
    I(isize),
    F(f64),
    Nodes(Vec<Node>),
    LetVar(NodeValueLet),
}
impl NodeValue {
    pub fn to_string(&self) -> String {
        match self {
            NodeValue::Empty => String::from("Empty"),
            NodeValue::S(v) => format!("{}", v),
            NodeValue::I(v) => format!("{}", v),
            NodeValue::F(v) => format!("{}", v),
            _ => String::from(""),
        }
    }
}

#[derive(Debug,Clone)]
pub struct NodeValueLet {
    pub varname: String,
    pub value_node: Vec<Node>,
}
impl NodeValueLet {
    pub fn new(varname: String, value_node: Vec<Node>) -> Self {
        Self {
            varname,
            value_node,
        }
    }
}

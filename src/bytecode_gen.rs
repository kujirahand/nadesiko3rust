//! バイトコードを生成

use crate::node::*;
use std::collections::HashMap;

/// stack machine byte code
#[derive(Debug,Clone)]
pub enum Bytecode {
    Nop,
    DebugInfo(i32, i32, i32, String), // (start, end, fileno, comment)
    Label(String),
    ConstInt(isize), // push:1
    ConstFloat(f64), // push:1
    ConstBool(bool), // push:1
    ConstStr(usize), // push:1=string_pool[index]
    LetVarGlobal(usize), // pop:1, key=string_pool[index]
    GetVarGlobal(usize), // push:1, key = string_pool[index]
    LetVarLocal(usize), // pop:1
    GetVarLocal(usize), // push:1
    JUMP(isize),
    CALL(isize),
    RET,
    CalcPlus,
    CalcMinus,
    CalcMul,
    CalcDiv,
    CalcMod,
    CalcEq,
    CaclGt,
    CalgGtEq,
    CalcLt,
    CalcLtEq,
    CreateArray, // push:1
    RefArray(usize), // pop:1 push:1
    LetArray(usize, usize), // pop:1
}

#[derive(Debug,Clone)]
pub struct BytecodeItems {
    pub index: usize,
    pub codes: Vec<Bytecode>,
    pub string_pool: Vec<String>,
    pub labels: HashMap<String, isize>,
    pub errors: Vec<String>,
    pub stack: Vec<NodeValue>,
    pub global_vars: HashMap<String, NodeValue>,
}

impl BytecodeItems {
    pub fn new() -> Self {
        Self {
            index: 0,
            codes: vec![],
            string_pool: vec![],
            labels: HashMap::new(),
            errors: vec![],
            stack: vec![],
            global_vars: HashMap::new(),
        }
    }
    pub fn get_string_id(&mut self, s: &str) -> usize {
        // 既に登録されている？
        for (i, ss) in self.string_pool.iter().enumerate() {
            if s.eq(ss) { return i; }
        }
        // 文字列プールに追加してIDを返す
        let id = self.string_pool.len();
        self.string_pool.push(String::from(s));
        id
    }
    pub fn get_string(&self, id: usize) -> String {
        self.string_pool[id].clone()
    }
    pub fn add_debug_info(&mut self, node: &Node, comment: String) {
        self.codes.push(Bytecode::DebugInfo(node.pos.start, node.pos.end, node.pos.fileno, comment));
    }
}

pub fn generate(nodes: &Vec<Node>) -> Result<BytecodeItems, String> {
    let mut items = BytecodeItems::new();
    generate_nodes(&mut items, nodes);
    if items.errors.len() > 0 { return Err(items.errors.join("\n")); }
    Ok(items)
}

fn generate_nodes(items: &mut BytecodeItems, nodes: &Vec<Node>) {
    for node in nodes.iter() {
        generate_node(items, node);
        if items.errors.len() > 0 { break; }
    }
}

fn generate_node(items: &mut BytecodeItems, node: &Node) {
    match node.kind {
        NodeKind::Int => items.codes.push(Bytecode::ConstInt(node.value.to_int(0))),
        NodeKind::Number => items.codes.push(Bytecode::ConstFloat(node.value.to_float(0.0))),
        NodeKind::String => {
            let id = items.get_string_id(&node.value.to_string());
            items.codes.push(Bytecode::ConstStr(id));
        },
        NodeKind::LetVarGlobal => {
            let params = match &node.value {
                NodeValue::LetVar(p) => p,
                _ => { items.errors.push(format!("({})代入文の生成でエラー", node.pos.start)); return; }
            };
            let var_name = &params.var_info.name;
            items.add_debug_info(node, format!("変数『{}』への代入", var_name));
            generate_nodes(items, &params.value_node);
            let var_id = items.get_string_id(&params.var_info.name);
            items.codes.push(Bytecode::LetVarGlobal(var_id));
        },
        _ => { println!("[ERROR] not yet implements: {:?}", node); }
    }
}


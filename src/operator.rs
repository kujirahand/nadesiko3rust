use crate::node::*;

// @see https://github.com/kujirahand/nadesiko3/blob/a80fd6074dc171cfae41457d1cd4c390a5aa43a4/src/nako_parser_const.js

// 演算子の優先順位を定義
pub fn get_priority(c: char) -> i8 {
    match c {
        '(' => 60,
        '^' => 50,
        // mul, div
        '*' => 40, 
        '/' => 40,
        '%' => 40,
        // plus, minus
        '+' => 30,
        '結' => 30, // 文字列の加算
        '-' => 30,
        // comp
        '>' => 20,
        '≧' => 20,
        '<' => 20,
        '≦' => 20,
        '=' => 20,
        '≠' => 20,
        // or and
        '&' => 10,
        '|' => 10,
        _ => 127,
    }
}

pub fn get_node_priority(node_v: &Node) -> i8 {
    match node_v.kind {
        NodeKind::Operator => {
            match &node_v.value {
                NodeValue::Operator(op) => {
                    get_priority(op.flag)
                },
                _ => 127,
            }
        },
        NodeKind::Bool => 127,
        NodeKind::Int => 127,
        NodeKind::Number => 127,
        NodeKind::String => 127,
        NodeKind::GetVar => 127,
        NodeKind::CallSysFunc => 5,
        NodeKind::CallUserFunc => 5,
        _ => 127,
    }
}

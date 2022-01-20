use crate::node::*;

// @see https://github.com/kujirahand/nadesiko3/blob/a80fd6074dc171cfae41457d1cd4c390a5aa43a4/src/nako_parser_const.js

// 演算子の優先順位を定義
pub fn get_priority(c: char) -> i8 {
    match c {
        '(' => 6,
        '^' => 5,
        // mul, div
        '*' => 4, 
        '/' => 4,
        '%' => 4,
        // plus, minus
        '+' => 3,
        '-' => 3,
        // comp
        '>' => 2,
        '≧' => 2,
        '<' => 2,
        '≦' => 2,
        '=' => 2,
        '≠' => 2,
        // or and
        '&' => 1,
        '|' => 1,
        _ => 127,
    }
}

pub fn get_node_priority(node_v: &Node) -> i8 {
    match node_v.kind {
        NodeKind::Operator => {
            match &node_v.value {
                NodeValue::Operator(op) => get_priority(op.flag),
                _ => 127,
            }
        },
        _ => 127,
    }
}

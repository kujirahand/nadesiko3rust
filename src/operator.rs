//! 演算子の優先順位を定義

use crate::node::*;

//// 演算における値の優先順位
const OP_PRIORITY_VALUE: i8 = 100;
//// 演算における関数の優先順位
const OP_PRIORTY_FUNCTION: i8 = 5;

/// 演算子の優先順位を定義
// (memo) なでしこv3 オリジナルの優先順位 <https://github.com/kujirahand/nadesiko3/blob/a80fd6074dc171cfae41457d1cd4c390a5aa43a4/src/nako_parser_const.js>
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
        _ => OP_PRIORITY_VALUE,
    }
}

/// Nodeに対して優先順位を演算の取得する関数
pub fn get_node_priority(node_v: &Node) -> i8 {
    match node_v.kind {
        NodeKind::Operator => {
            match &node_v.value {
                NodeValue::Operator(op) => {
                    get_priority(op.flag)
                },
                _ => OP_PRIORITY_VALUE,
            }
        },
        NodeKind::Bool => OP_PRIORITY_VALUE,
        NodeKind::Int => OP_PRIORITY_VALUE,
        NodeKind::Number => OP_PRIORITY_VALUE,
        NodeKind::String => OP_PRIORITY_VALUE,
        NodeKind::GetVarGlobal => OP_PRIORITY_VALUE,
        NodeKind::CallSysFunc => OP_PRIORTY_FUNCTION,
        NodeKind::CallUserFunc => OP_PRIORTY_FUNCTION,
        _ => OP_PRIORITY_VALUE,
    }
}

// 走者 - Vec<Node>を順に実行
use crate::{tokenizer, parser};
use crate::node::*;

pub fn indent_str(num: usize) -> String {
    let mut s = String::new();
    for _ in 0..num {
        s.push_str("    ");
    }
    s
}

pub fn run_nodes(ctx: &mut NodeContext, nodes: &Vec<Node>) -> NodeValue {
    ctx.callstack_level += 1;
    let nodes_len = nodes.len();
    let mut result = NodeValue::Empty;
    let mut index = 0;
    while index < nodes_len {
        let cur:&Node = &nodes[index];
        println!("[run]({:02}) {}{}", ctx.index, indent_str(ctx.callstack_level-1), cur.to_string());
        match cur.kind {
            NodeKind::Comment => {},
            NodeKind::Let => result = run_let(ctx, cur),
            NodeKind::Int => result = cur.value.clone(),
            NodeKind::GetVar => result = run_get_var(ctx, cur),
            NodeKind::Operator => result = run_operator(ctx, cur),
            NodeKind::DebugPrint => result = run_debug_print(ctx, cur),
            _ => {
                println!("Not implement:{:?}", cur);
            }
        }
        index += 1;
    }
    ctx.callstack_level -= 1;
    result
}

fn run_debug_print(ctx: &mut NodeContext, node: &Node) -> NodeValue {
    let arg_nodes: &Vec<Node> = match &node.value {
        NodeValue::Nodes(ref nodes, _) => nodes,
        _ => return NodeValue::Empty,
    };
    let v = run_nodes(ctx, arg_nodes);
    println!("[DEBUG] {}", v.to_string());
    v
}

fn run_let(ctx: &mut NodeContext, node: &Node) -> NodeValue {
    let let_value: &NodeValueLet = match &node.value {
        NodeValue::LetVar(ref let_value) => let_value,
        _ => return NodeValue::Empty,
    };
    let value_node:&Vec<Node> = &let_value.value_node;
    let value = run_nodes(ctx, value_node);
    let info: &NodeVarInfo = &let_value.var_info;
    ctx.scopes[info.level].var_values[info.no] = value.clone();
    value
}

fn run_get_var(ctx: &mut NodeContext, node: &Node) -> NodeValue {
    let var_info: &NodeVarInfo = match &node.value {
        NodeValue::GetVar(ref var_info) => var_info,
        _ => return NodeValue::Empty,
    };
    match ctx.get_var_value(var_info) {
        Some(v) => v,
        None => NodeValue::Empty
    }
}

fn run_operator(ctx: &mut NodeContext, node: &Node) -> NodeValue {
    let (nodes, op_str) = match &node.value {
        NodeValue::Nodes(nodes, label) => (nodes, label),
        _ => return NodeValue::Empty,
    };
    let right = run_nodes(ctx, &vec![nodes[1].clone()]);
    let left = run_nodes(ctx, &vec![nodes[0].clone()]);
    if op_str.eq("") {
        // todo: runtime error
        return NodeValue::Empty;
    }
    let op_char:char = op_str.chars().nth(0).unwrap_or('\0');
    match op_char {
        '+' => NodeValue::calc_plus(left, right),
        '-' => NodeValue::calc_minus(left, right),
        '*' => NodeValue::calc_mul(left, right),
        '/' => NodeValue::calc_div(left, right),
        '%' => NodeValue::calc_mod(left, right),
        _ => NodeValue::Empty,
    }
}

// -----------------------------------------------
// eval
// -----------------------------------------------
pub fn eval_str(code: &str) -> NodeValue {
    let tokens = tokenizer::tokenize(code);
    let mut p = parser::Parser::new();
    p.parse(tokens, "run.nako3");
    let mut ctx = p.clone_context();
    let result = run_nodes(&mut ctx, &p.nodes);
    result
}

#[cfg(test)]
mod test_runner {
    use super::*;

    #[test]
    fn test_if() {
        //let res = run_str("N=1;もしN=1ならば\n「OK」とデバッグ表示;\n違えば\n「NG」とデバッグ表示\nここまで;");
        //assert_eq!(res.to_int(0), 123);
    }
    #[test]
    fn test_debug_print() {
        let res = eval_str("123とデバッグ表示");
        assert_eq!(res.to_int(0), 123);
    }
    #[test]
    fn test_calc() {
        let res = eval_str("1+2とデバッグ表示");
        assert_eq!(res.to_int(0), 3);
    }
}

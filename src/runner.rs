// 走者 - Vec<Node>を順に実行
use crate::{tokenizer, parser};
use crate::node::*;
use crate::sys_function_debug;
use crate::sys_function;

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
        println!("[RUN]({:02}) {}{}", ctx.index, indent_str(ctx.callstack_level-1), cur.to_string());
        match cur.kind {
            NodeKind::Comment => {},
            NodeKind::Let => result = run_let(ctx, cur),
            NodeKind::Int => result = cur.value.clone(),
            NodeKind::Number => result = cur.value.clone(),
            NodeKind::String => result = cur.value.clone(),
            NodeKind::StringEx => result = cur.value.clone(), // TODO: 変数の展開
            NodeKind::GetVar => result = run_get_var(ctx, cur),
            NodeKind::Operator => result = run_operator(ctx, cur),
            NodeKind::CallSysFunc => result = run_call_sysfunc(ctx, cur),
            _ => {
                println!("[エラー] runner未実装のノード :{:?}", cur);
            }
        }
        index += 1;
    }
    ctx.callstack_level -= 1;
    result
}

fn run_call_sysfunc(ctx: &mut NodeContext, node: &Node) -> NodeValue {
    let mut args: Vec<NodeValue> = vec![];
    let func_no = match &node.value {
        NodeValue::SysFunc(no, nodes) => {
            for n in nodes.iter() {
                let v = run_nodes(ctx, &vec![n.clone()]);
                args.push(v);
            }
            *no
        }
        _ => return NodeValue::Empty,
    };
    let info:&SysFuncInfo = &ctx.sysfuncs[func_no];
    (info.func)(ctx, args)
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
    let op = match &node.value {
        NodeValue::Operator(op) => op,
        _ => return NodeValue::Empty,
    };
    let right = run_nodes(ctx, &vec![op.nodes[1].clone()]);
    let left = run_nodes(ctx, &vec![op.nodes[0].clone()]);
    match op.flag {
        '(' => left,
        '+' => NodeValue::calc_plus(&left, &right),
        '-' => NodeValue::calc_minus(&left, &right),
        '*' => NodeValue::calc_mul(&left, &right),
        '/' => NodeValue::calc_div(&left, &right),
        '%' => NodeValue::calc_mod(&left, &right),
        _ => NodeValue::Empty,
    }
}

// -----------------------------------------------
// eval
// -----------------------------------------------
#[derive(Debug,Clone)]
pub struct RunOption {
    pub use_sysfunc: bool,
    pub debug: bool,
}
impl RunOption {
    pub fn normal() -> Self {
        Self { use_sysfunc: true, debug: false }
    }
    pub fn simple() -> Self {
        Self { use_sysfunc: false, debug: true }
    }
}
pub fn eval(code: &str, options: RunOption) -> Result<NodeValue,String> {
    // 意味解析器を初期化
    let mut p = parser::Parser::new();
    if options.use_sysfunc {
        sys_function::register(&mut p.context);
    } else {
        sys_function_debug::register(&mut p.context);
    }
    // 字句解析
    let tokens = tokenizer::tokenize(code);
    // 意味解析
    let nodes = match p.parse(tokens, "eval.nako3") {
        Ok(nodes) => nodes,
        Err(e) => { return Err(e); }
    };
    let result = run_nodes(&mut p.context, &nodes);
    Ok(result)
}
pub fn eval_str(code: &str) -> String {
    match eval(code, RunOption::normal()) {
        Ok(v) => v.to_string(),
        Err(e) => format!("!!{}", e),
    }
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
    fn test_print() {
        let res = eval("123と表示", RunOption::normal());
        assert_eq!(res, Ok(NodeValue::I(123)));
        let res = eval_str("「穏やかな心は体に良い」と表示");
        assert_eq!(res, String::from("穏やかな心は体に良い"));
    }

    #[test]
    fn test_debug_print() {
        let res = eval("123と表示", RunOption::simple());
        assert_eq!(res, Ok(NodeValue:I(123)));
    }
    #[test]
    fn test_calc() {
        let res = eval_str("1+2と表示");
        assert_eq!(res, String::from("3"));
        let res = eval_str("1+2*3と表示");
        assert_eq!(res, String::from("7"));
        let res = eval_str("(1+2)*3と表示");
        assert_eq!(res, String::from("9"));
    }
}

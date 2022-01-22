// 走者 - Vec<Node>を順に実行
use crate::{tokenizer, parser};
use crate::node::*;
use crate::context::*;
use crate::sys_function_debug;
use crate::sys_function;

pub fn indent_str(num: usize) -> String {
    let mut s = String::new();
    for _ in 0..num {
        s.push_str("    ");
    }
    s
}

pub fn run_nodes(ctx: &mut NodeContext, nodes: &Vec<Node>) -> Result<NodeValue, String> {
    ctx.callstack_level += 1;
    let nodes_len = nodes.len();
    let mut result = NodeValue::Empty;
    let mut index = 0;
    while index < nodes_len {
        if ctx.has_error() { return Err(ctx.get_error_str()); }
        let cur:&Node = &nodes[index];
        println!("[RUN]({:02}) {}{}", index, indent_str(ctx.callstack_level-1), cur.to_string());
        match cur.kind {
            NodeKind::Comment => {},
            NodeKind::Let => result = run_let(ctx, cur),
            NodeKind::Int => result = cur.value.clone(),
            NodeKind::Bool => result = cur.value.clone(),
            NodeKind::Number => result = cur.value.clone(),
            NodeKind::String => result = cur.value.clone(),
            NodeKind::StringEx => result = cur.value.clone(), // TODO: 変数の展開
            NodeKind::GetVar => result = run_get_var(ctx, cur),
            NodeKind::Operator => result = run_operator(ctx, cur),
            NodeKind::CallSysFunc => result = run_call_sysfunc(ctx, cur),
            NodeKind::NodeList => {
                result = match run_nodes(ctx, &cur.value.to_nodes()) {
                    Ok(value) => value,
                    Err(_) => return Err(ctx.get_error_str()),
                };
            },
            NodeKind::If => {
                let nodes = cur.value.to_nodes();
                let cond: Node = (&nodes[0]).clone();
                let true_node: Node = (&nodes[1]).clone();
                let false_node: Node = (&nodes[2]).clone();
                match run_nodes(ctx, &vec![cond]) {
                    Err(_) => return Err(ctx.get_error_str()),
                    Ok(v) => {
                        if v.to_bool() {
                            result = run_nodes(ctx, &vec![true_node]).unwrap_or(NodeValue::Empty);
                        } else {
                            result = run_nodes(ctx, &vec![false_node]).unwrap_or(NodeValue::Empty);
                        }
                    }
                }
            }
            _ => {
                println!("[エラー] runner未実装のノード :{:?}", cur);
            }
        }
        index += 1;
    }
    ctx.callstack_level -= 1;
    Ok(result)
}

fn run_call_sysfunc(ctx: &mut NodeContext, node: &Node) -> NodeValue {
    let mut args: Vec<NodeValue> = vec![];
    let func_no = match &node.value {
        NodeValue::SysFunc(no, nodes) => {
            for n in nodes.iter() {
                let v = match run_nodes(ctx, &vec![n.clone()]) {
                    Ok(v) => v,
                    Err(_) => return NodeValue::Empty,
                };
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
    let value = match run_nodes(ctx, value_node) {
        Ok(v) => v,
        Err(_) => NodeValue::Empty,
    };
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
    let right = run_nodes(ctx, &vec![op.nodes[1].clone()]).unwrap_or(NodeValue::Empty);
    let left = run_nodes(ctx, &vec![op.nodes[0].clone()]).unwrap_or(NodeValue::Empty);
    match op.flag {
        '(' => left,
        '!' => NodeValue::B(!left.to_bool()),
        '+' => NodeValue::calc_plus(&left, &right),
        '結' => NodeValue::calc_plus_str(&left, &right), // 文字列加算
        '|' => NodeValue::calc_or(&left, &right), // または
        '&' => NodeValue::calc_and(&left, &right), // かつ
        '-' => NodeValue::calc_minus(&left, &right),
        '*' => NodeValue::calc_mul(&left, &right),
        '/' => NodeValue::calc_div(&left, &right),
        '%' => NodeValue::calc_mod(&left, &right),
        '=' => NodeValue::calc_eq(&left, &right),
        '≠' => NodeValue::calc_noteq(&left, &right),
        '>' => NodeValue::calc_gt(&left, &right),
        '≧' => NodeValue::calc_gteq(&left, &right),
        '<' => NodeValue::calc_lt(&left, &right),
        '≦' => NodeValue::calc_lteq(&left, &right),
        '^' => NodeValue::calc_pow(&left, &right),
        _ => {
            println!("[実行時エラー]未実装の演算子記号:『{}』", op.flag);
            NodeValue::Empty
        },
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
    run_nodes(&mut p.context, &nodes)
}

pub fn eval_str(code: &str) -> String {
    match eval(code, RunOption::normal()) {
        Ok(v) => v.to_string(),
        Err(e) => format!("!!{}", e),
    }
}
pub fn eval_simple_str(code: &str) -> String {
    match eval(code, RunOption::simple()) {
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
        let res = eval_str("123と表示");
        assert_eq!(res, String::from("123"));
        let res = eval_str("「穏やかな心は体に良い」と表示");
        assert_eq!(res, String::from("穏やかな心は体に良い"));
    }

    #[test]
    fn test_debug_print() {
        let res = eval("123と表示", RunOption::simple());
        assert_eq!(res.unwrap_or(NodeValue::Empty).to_int(0), 123);
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
    #[test]
    fn test_calc_check_operator() {
        let res = eval_str("1+2と表示");
        assert_eq!(res, String::from("3"));
        let res = eval_str("3*2と表示");
        assert_eq!(res, String::from("6"));
        let res = eval_str("3×2と表示");
        assert_eq!(res, String::from("6"));
        let res = eval_str("1÷2と表示");
        assert_eq!(res, String::from("0.5"));
        let res = eval_str("1/2と表示");
        assert_eq!(res, String::from("0.5"));
        let res = eval_str("2^3と表示");
        assert_eq!(res, String::from("8"));
        let res = eval_str("10%3と表示");
        assert_eq!(res, String::from("1"));
        let res = eval_str("10>3と表示");
        assert_eq!(res, String::from("真"));
        let res = eval_str("10<3と表示");
        assert_eq!(res, String::from("偽"));
        let res = eval_str("5>=5と表示");
        assert_eq!(res, String::from("真"));
        let res = eval_str("5<5と表示");
        assert_eq!(res, String::from("偽"));
        let res = eval_str("5<=5と表示");
        assert_eq!(res, String::from("真"));
        let res = eval_str("真&&真と表示");
        assert_eq!(res, String::from("真"));
        let res = eval_str("真||偽と表示");
        assert_eq!(res, String::from("真"));
    }

}

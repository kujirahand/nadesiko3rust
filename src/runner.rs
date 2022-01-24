// 走者 - Vec<Node>を順に実行
use crate::{tokenizer, parser};
use crate::node::*;
use crate::context::*;
use crate::sys_function_debug;
use crate::sys_function;

pub fn run_node(ctx: &mut NodeContext, cur: &Node) -> Option<NodeValue> {
    let mut result = NodeValue::Empty;
    match cur.kind {
        NodeKind::Nop => {},
        NodeKind::Comment => {},
        NodeKind::Let => result = run_let(ctx, cur),
        NodeKind::Int => result = cur.value.clone(),
        NodeKind::Bool => result = cur.value.clone(),
        NodeKind::Number => result = cur.value.clone(),
        NodeKind::String => result = cur.value.clone(),
        NodeKind::GetVar => result = run_get_var(ctx, cur).unwrap_or(NodeValue::Empty),
        NodeKind::Operator => result = run_operator(ctx, cur),
        NodeKind::CallSysFunc => result = run_call_sysfunc(ctx, cur),
        NodeKind::NodeList => {
            result = match run_nodes(ctx, &cur.value.to_nodes()) {
                Ok(value) => value,
                Err(_) => return None,
            };
        },
        NodeKind::If => match run_if(ctx, cur) { Some(v) => result = v, None => {}},
        NodeKind::Kai => match run_kai(ctx, cur) { Some(v) => result = v, None => {}},
        NodeKind::Break => { ctx.try_break = Some(ctx.callstack_level) },
        NodeKind::Continue => { ctx.try_continue = Some(ctx.callstack_level) },
        // _ => { println!("[エラー] runner未実装のノード :{:?}", cur); return None; }
    }
    Some(result)
}

pub fn run_kai(ctx: &mut NodeContext, cur: &Node) -> Option<NodeValue> {
    let nodes = cur.value.to_nodes();
    let kaisu_node = &nodes[0];
    let body_node = &nodes[1];
    let kaisu = run_node(ctx, kaisu_node).unwrap_or(NodeValue::I(0));
    let mut result = None;
    for i in 0..kaisu.to_int(0) {
        ctx.scopes.set_value(1, "回数", NodeValue::I(i + 1));
        result = run_node(ctx, body_node);
        // 抜けるの処理
        if ctx.try_break != None {
            ctx.try_break = None;
            break;
        }
        // 続けるの処理
        if ctx.try_continue != None {
            ctx.try_continue = None;
            continue;
        }
    }
    result
}

pub fn run_if(ctx: &mut NodeContext, cur: &Node) -> Option<NodeValue> {
    let nodes = cur.value.to_nodes();
    let cond: &Node = &nodes[0];
    let true_node: &Node = &nodes[1];
    let false_node: &Node = &nodes[2];
    match run_node(ctx, cond) {
        None => return None,
        Some(cond_v) => {
            if cond_v.to_bool() {
                return run_node(ctx, true_node);
            } else {
                return run_node(ctx, false_node);
            }
        }
    }
}


pub fn run_nodes(ctx: &mut NodeContext, nodes: &Vec<Node>) -> Result<NodeValue, String> {
    ctx.callstack_level += 1;
    let nodes_len = nodes.len();
    let mut result = NodeValue::Empty;
    let mut index = 0;
    while index < nodes_len {
        if ctx.has_error() { return Err(ctx.get_error_str()); }
        if ctx.try_continue != None { return Ok(NodeValue::Empty); }
        if ctx.try_break != None { return Ok(NodeValue::Empty); }
        let cur:&Node = &nodes[index];
        println!("[RUN]({:02}) {}{}", index, indent_str(ctx.callstack_level-1), cur.to_string());
        if let Some(v) = run_node(ctx, cur) { result = v; }
        index += 1;
    }
    ctx.callstack_level -= 1;
    Ok(result)
}

fn run_call_sysfunc(ctx: &mut NodeContext, node: &Node) -> NodeValue {
    let mut args: Vec<NodeValue> = vec![];
    let func_no = match &node.value {
        NodeValue::SysFunc(func_name, no, nodes) => {
            for n in nodes.iter() {
                let v = match run_nodes(ctx, &vec![n.clone()]) {
                    Ok(v) => v,
                    Err(err) => {
                        ctx.throw_error(
                            NodeErrorKind::RuntimeError, NodeErrorLevel::Error, 
                            format!("『{}』の呼び出しでエラー。{}", func_name, err), 
                            node.line, node.fileno);
                        return NodeValue::Empty;
                    }
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
    let value = run_nodes(ctx, value_node).unwrap_or(NodeValue::Empty);
    ctx.scopes.set_value(1, &let_value.var_name, value.clone());
    value
}

fn run_get_var(ctx: &mut NodeContext, node: &Node) -> Option<NodeValue> {
    let var_info: &NodeVarInfo = match &node.value {
        NodeValue::GetVar(ref var_info) => var_info,
        _ => return None,
    };
    ctx.get_var_value(var_info)
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

pub fn indent_str(num: usize) -> String {
    let mut s = String::new();
    for _ in 0..num {
        s.push_str("    ");
    }
    s
}

#[cfg(test)]
mod test_runner {
    use super::*;

    #[test]
    fn test_if() {
        let res = eval_str("N=1;もしN=1ならば\n「OK」と表示;\n違えば\n「NG」と表示\nここまで;");
        assert_eq!(res, "OK");
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
        let res = eval_str("(1==1)&&(2==2)と表示");
        assert_eq!(res, String::from("真"));
    }
    #[test]
    fn test_string_ex() {
        let res = eval_str("A=123;「A={A}」と表示");
        assert_eq!(res, "A=123");
    }

}

//! インタプリタ Node を順に実行する
// 走者 - Vec<Node>を順に実行
use crate::{tokenizer, parser};
use crate::node::*;
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
        NodeKind::CallUserFunc => result = run_call_userfunc(ctx, cur),
        NodeKind::NodeList => {
            result = match run_nodes(ctx, &cur.value.to_nodes()) {
                Ok(value) => value,
                Err(_) => return None,
            };
        },
        NodeKind::If => match run_if(ctx, cur) { Some(v) => result = v, None => {}},
        NodeKind::Kai => match run_kai(ctx, cur) { Some(v) => result = v, None => {}},
        NodeKind::For => match run_for(ctx, cur) { Some(v) => result = v, None => {}},
        NodeKind::Break => { ctx.try_break = Some(ctx.callstack_level) },
        NodeKind::Continue => { ctx.try_continue = Some(ctx.callstack_level) },
        NodeKind::Return => result = run_return(ctx, cur),
        // _ => { println!("[エラー] runner未実装のノード :{:?}", cur); return None; }
    }
    Some(result)
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
        if ctx.try_return != None { return Ok(NodeValue::Empty); }
        let cur:&Node = &nodes[index];
        if ctx.debug_mode {
            println!("[RUN:{:2}] {}{}", cur.line, indent_str(ctx.callstack_level-1), cur.to_string());
        }
        if let Some(v) = run_node(ctx, cur) { result = v; }
        index += 1;
    }
    ctx.callstack_level -= 1;
    Ok(result)
}


pub fn run_return(ctx: &mut NodeContext, cur: &Node) -> NodeValue {
    match &cur.value {
        NodeValue::NodeList(nodes) => {
            let node = &nodes[0];
            let result = run_node(ctx, node).unwrap_or(NodeValue::Empty);
            ctx.scopes.set_value_local_scope("それ", result.clone());
            ctx.try_return = Some(ctx.callstack_level);
            println!("*** RETUEN ***");
            result
        },
        _ => NodeValue::Empty,
    }
}

pub fn run_for(ctx: &mut NodeContext, cur: &Node) -> Option<NodeValue> {
    let nodes = cur.value.to_nodes();
    let loop_node = &nodes[0];
    let kara_node = &nodes[1];
    let made_node = &nodes[2];
    let body_node = &nodes[3];
    let kara_v = run_node(ctx, &kara_node).unwrap_or(NodeValue::Empty);
    let made_v = run_node(ctx, &made_node).unwrap_or(NodeValue::Empty);
    let mut result = None;
    for i in kara_v.to_int(0)..=made_v.to_int(0) {
        if loop_node.kind == NodeKind::GetVar {
            match &loop_node.value {
                NodeValue::GetVar(info) => {
                    let name: String = info.name.clone().unwrap_or(String::new());
                    ctx.scopes.set_value_local_scope(&name, NodeValue::I(i));
                },
                _ => {},
            }
        }
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
        // 戻るの処理
        if ctx.try_return != None {
            break;
        }
    }
    result
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
        // 戻るの処理
        if ctx.try_return != None {
            break;
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
    let result = (info.func)(ctx, args);
    match result {
        Some(value) => {
            ctx.scopes.set_value_local_scope("それ", value.clone());
            value
        },
        None => return NodeValue::Empty,
    }
}

fn run_call_userfunc(ctx: &mut NodeContext, node: &Node) -> NodeValue {
    // 関数呼び出しの引数を得る
    let (func_name, func_no, arg_nodes) = match &node.value {
        NodeValue::SysFunc(func_name, no, nodes) => (func_name, *no, nodes),
        _ => return NodeValue::Empty,
    };
    if ctx.debug_mode {
        println!("[DEBUG] --- run_call_userfunc:{} ---", func_name);
    }
    // 関数を得る
    let info = NodeVarInfo { level: 1, no: func_no, name: None };
    let func = match ctx.scopes.get_var_value(&info) { // 関数本体
        Some(v) => v,
        None => return NodeValue::Empty,
    };
    let meta = match ctx.scopes.get_var_meta(&info) { // メタ情報
        Some(v) => v,
        None => return NodeValue::Empty,
    };
    // 関数の引数定義を得る
    let func_args = match meta.kind {
        NodeVarKind::UserFunc(args) => args,
        _ => return NodeValue::Empty,
    };
    // 関数スコープを作り、ローカル変数を登録する
    let mut scope = NodeScope::new();
    // 関数の引数を得る
    for (i, n) in arg_nodes.iter().enumerate() {
        match run_nodes(ctx, &vec![n.clone()]) {
            Ok(val) => {
                let name = &func_args[i].name;
                scope.set_var(name, val);
            },
            Err(err) => {
                ctx.throw_error(
                    NodeErrorKind::RuntimeError, NodeErrorLevel::Error, 
                    format!("『{}』の呼び出しでエラー。{}", func_name, err), 
                    node.line, node.fileno);
                return NodeValue::Empty;
            }
        };
    }
    // 関数を実行
    ctx.scopes.push_local(scope);
    let tmp_return_level = ctx.return_level;
    ctx.return_level = ctx.callstack_level;
    match func {
        NodeValue::SysFunc(name, _no, nodes) => {
            // println!("@@@CALL:{}", nodes_to_string(&nodes, "\n"));
            match run_nodes(ctx, &nodes) {
                Ok(v) => v,
                Err(e) => {
                    ctx.throw_runtime_error(format!("『{}』の呼び出しでエラー。{}", name, e), node.line, node.fileno);
                    NodeValue::Empty
                }
            };
        },
        _ => {},    
    };
    let func_scope = ctx.scopes.pop_local().unwrap_or(NodeScope::new());
    if let Some(_level) = ctx.try_return {
        ctx.try_return = None;
    }
    ctx.return_level = tmp_return_level;
    let result = func_scope.get_var("それ");
    // println!("*** 関数のスコープ={:?}", func_scope);
    ctx.scopes.set_value_local_scope("それ", result.clone());
    result
}

fn run_let(ctx: &mut NodeContext, node: &Node) -> NodeValue {
    let let_value: &NodeValueLet = match &node.value {
        NodeValue::LetVar(ref let_value) => let_value,
        _ => return NodeValue::Empty,
    };
    let value_node:&Vec<Node> = &let_value.value_node;
    let value = run_nodes(ctx, value_node).unwrap_or(NodeValue::Empty);
    let info = let_value.var_info.clone();
    let name = info.name.unwrap_or(String::new()).clone();
    ctx.scopes.set_value(info.level, &name, value.clone());
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
        s.push_str("  ");
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
        let res = eval_str("1+2*3と表示");
        assert_eq!(res, String::from("7"));
        let res = eval_str("1*2+3と表示");
        assert_eq!(res, String::from("5"));
        let res = eval_str("5%2+1と表示");
        assert_eq!(res, String::from("2"));
        let res = eval_str("5%2=1と表示");
        assert_eq!(res, String::from("真"));
    }
    #[test]
    fn test_string_ex() {
        let res = eval_str("A=123;「A={A}」と表示");
        assert_eq!(res, "A=123");
    }

    #[test]
    fn test_let_eval() {
        let res = eval_str("A=1に2を足す。Aを表示。");
        assert_eq!(res, "3");
        let res = eval_str("A=2に3を掛ける。Aを表示。");
        assert_eq!(res, "6");
    }

    #[test]
    fn test_calc_long() {
        let res = eval_str("(5から3を引く)を表示。");
        assert_eq!(res, "2");
        let res = eval_str("5*2+2+3を表示。");
        assert_eq!(res, "15");
    }
}

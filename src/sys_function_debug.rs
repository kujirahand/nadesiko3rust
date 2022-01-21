//------------------------------------
// デバッグ用の関数
//------------------------------------
// (用途) 標準関数を全部足すとデバッグがやりづらい時に使う。最低限の関数定義
use crate::node::*;
use crate::context::*;

pub fn register(ctx: &mut NodeContext) {
    ctx.add_sysfunc("表示", sysargs(&[&["を", "と"]]), sys_debug_print);
    ctx.add_sysfunc("足", sysargs(&[&["と"], &["を"]]), sys_debug_add);
}

fn sys_debug_print(_: &mut NodeContext, args: Vec<NodeValue>) -> NodeValue {
    let s = if args.len() > 0 { args[0].to_string() } else { String::from("<表示内容がありません>") };
    println!("[DEBUG] {}", s);
    NodeValue::S(s)
}

fn sys_debug_add(_: &mut NodeContext, args: Vec<NodeValue>) -> NodeValue {
    if args.len() < 2 { return NodeValue::Empty; }
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_plus(&a, &b);
    res
}
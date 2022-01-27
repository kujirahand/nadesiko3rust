//! なでしこの関数を定義したもの(デバッグ用)
//------------------------------------
// デバッグ用の関数
//------------------------------------
// (用途) 標準関数を全部足すとデバッグがやりづらい時に使う。最低限の関数定義
use crate::node::*;

pub fn register(ctx: &mut NodeContext) {
    ctx.add_sysfunc("表示", sysargs(&[&["を", "と"]]), sys_debug_print);
    ctx.add_sysfunc("足", sysargs(&[&["と","に"], &["を"]]), sys_debug_add);
}

fn sys_debug_print(_ctx: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let s = if args.len() > 0 { args[0].to_string() } else { String::from("<表示内容がありません>") };
    println!("[DEBUG] {}", s);
    Some(NodeValue::S(s))
}

fn sys_debug_add(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    if args.len() < 2 { return None; }
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_plus(&a, &b);
    Some(res)
}

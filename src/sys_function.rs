use crate::node::*;
use crate::context::*;

pub fn register(ctx: &mut NodeContext) {
    ctx.add_sysfunc("表示", sysargs(&[&["を", "と"]]), sys_print);
    ctx.add_sysfunc("足", sysargs(&[&["と"], &["を"]]), sys_add);
}

fn sys_print(_: &mut NodeContext, args: Vec<NodeValue>) -> NodeValue {
    let s = if args.len() > 0 { args[0].to_string() } else { String::from("<表示内容がありません>") };
    println!("{}", s);
    NodeValue::S(s)
} 

fn sys_add(_: &mut NodeContext, args: Vec<NodeValue>) -> NodeValue {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_plus(&a, &b);
    res
}

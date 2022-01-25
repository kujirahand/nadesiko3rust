use crate::node::*;

pub fn register(ctx: &mut NodeContext) {
    ctx.add_sysfunc("表示", sysargs(&[&["を", "と"]]), sys_print);
    ctx.add_sysfunc("足", sysargs(&[&["と","に"], &["を"]]), sys_add);
    ctx.add_sysfunc("引", sysargs(&[&["から"], &["を"]]), sys_sub);
    ctx.add_sysfunc("掛", sysargs(&[&["と","に"], &["を"]]), sys_mul);
    ctx.add_sysfunc("割", sysargs(&[&["を"], &["で"]]), sys_div);
    ctx.add_sysfunc("割余", sysargs(&[&["を"], &["で"]]), sys_mod);
}

fn sys_print(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let s = if args.len() > 0 { args[0].to_string() } else { String::from("<表示内容がありません>") };
    println!("{}", s);
    Some(NodeValue::S(s))
} 

fn sys_add(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_plus(&a, &b);
    Some(res)
}
fn sys_sub(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_minus(&a, &b);
    Some(res)
}
fn sys_mul(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_mul(&a, &b);
    Some(res)
}
fn sys_div(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_div(&a, &b);
    Some(res)
}
fn sys_mod(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_mod(&a, &b);
    Some(res)
}


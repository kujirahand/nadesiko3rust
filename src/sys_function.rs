use crate::node::*;


pub fn register(ctx: &mut NodeContext) {
    ctx.add_sysfunc("表示", sysargs(&[&["を", "と"]]), sys_print);
    ctx.add_sysfunc("足", sysargs(&[&["と"], &["を"]]), sys_add);
}

fn sys_print(_: &mut NodeContext, args: Vec<NodeValue>) -> NodeValue {
    let mut ss = String::new();
    for a in args.iter() {
        let s = a.to_string();
        ss.push_str(&s);
    }
    println!("{}", ss);
    NodeValue::Empty // Emptyにして変数「それ」を更新しない
}

fn sys_add(_: &mut NodeContext, args: Vec<NodeValue>) -> NodeValue {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_plus(&a, &b);
    res
}

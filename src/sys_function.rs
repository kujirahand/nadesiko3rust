//! なでしこの標準関数を定義したもの

use crate::node::*;
use std::{thread, time};

/// 関数をシステムに登録する
pub fn register(ctx: &mut NodeContext) {
    // 表示
    ctx.add_sysfunc("表示", sysargs(&[&["を", "と"]]), sys_print);
    // 四則演算
    ctx.add_sysfunc("足", sysargs(&[&["と","に"], &["を"]]), sys_add);
    ctx.add_sysfunc("引", sysargs(&[&["から"], &["を"]]), sys_sub);
    ctx.add_sysfunc("掛", sysargs(&[&["と","に"], &["を"]]), sys_mul);
    ctx.add_sysfunc("割", sysargs(&[&["を"], &["で"]]), sys_div);
    ctx.add_sysfunc("割余", sysargs(&[&["を"], &["で"]]), sys_mod);
    // 定数
    ctx.add_sysconst("永遠", NodeValue::B(true));
    ctx.add_sysconst("オン", NodeValue::B(true));
    ctx.add_sysconst("オフ", NodeValue::B(false));
    ctx.add_sysconst("OK", NodeValue::B(true));
    ctx.add_sysconst("NG", NodeValue::B(false));
    ctx.add_sysconst("改行", NodeValue::from_str("\n"));
    ctx.add_sysconst("タブ", NodeValue::from_str("\t"));
    ctx.add_sysconst("CR", NodeValue::from_str("\r"));
    ctx.add_sysconst("LF", NodeValue::from_str("\n"));
    ctx.add_sysconst("カッコ", NodeValue::from_str("「"));
    ctx.add_sysconst("カッコ閉", NodeValue::from_str("」"));
    ctx.add_sysconst("波カッコ", NodeValue::from_str("{"));
    ctx.add_sysconst("波カッコ閉", NodeValue::from_str("}"));
    ctx.add_sysconst("空", NodeValue::from_str(""));
    ctx.add_sysconst("PI", NodeValue::F(3.141592653589793));
    // 
    ctx.add_sysfunc("秒待", sysargs(&[&[""]]), sys_sleep);
}

fn sys_sleep(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let n = args[0].to_float(0.0);
    let sec = time::Duration::from_secs_f64(n);
    thread::sleep(sec);
    None
}

/// なでしこのシステム関数で画面表示
pub fn sys_print(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
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

#[cfg(test)]
mod test_runner {
    use super::super::runner::eval_str;

    #[test]
    fn test_add() {
        let res = eval_str("3に5を足して表示");
        assert_eq!(res, "8");
    }
    #[test]
    fn test_const() {
        let res = eval_str("カッコを表示");
        assert_eq!(res, "「");
    }
}
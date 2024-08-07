//! なでしこの標準関数を定義したもの

use crate::node::*;

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
    ctx.add_sysfunc("倍", sysargs(&[&["の", "を"], &[""]]), sys_mul);
    ctx.add_sysfunc("二乗", sysargs(&[&["の", "を"]]), sys_pow2);
    ctx.add_sysfunc("べき乗", sysargs(&[&["の"], &["の"]]), sys_pow);
    ctx.add_sysfunc("以上", sysargs(&[&["が"], &[""]]), sys_gteq);
    ctx.add_sysfunc("以下", sysargs(&[&["が"], &[""]]), sys_lteq);
    ctx.add_sysfunc("超", sysargs(&[&["が"], &[""]]), sys_gt);
    ctx.add_sysfunc("未満", sysargs(&[&["が"], &[""]]), sys_lt);
    ctx.add_sysfunc("等", sysargs(&[&["が"], &["と"]]), sys_eq);
    ctx.add_sysfunc("範囲内", sysargs(&[&["が"], &["から"], &["の", "までの"]]), sys_eq);
    // 型変換
    ctx.add_sysfunc("TYPEOF", sysargs(&[&["の"]]), sys_typeof);
    ctx.add_sysfunc("変数型確認", sysargs(&[&["の"]]), sys_typeof);
    ctx.add_sysfunc("INT", sysargs(&[&["の"]]), sys_toint);
    ctx.add_sysfunc("FLOAT", sysargs(&[&["の"]]), sys_tofloat);
    ctx.add_sysfunc("HEX", sysargs(&[&["の"]]), sys_hex);
    ctx.add_sysfunc("二進", sysargs(&[&["の"]]), sys_bin);
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
}

/// なでしこのシステム関数で画面表示
pub fn sys_print(ctx: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let s = if args.len() > 0 { args[0].to_string() } else { String::from("<表示内容がありません>") };
    ctx.println(&s);
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
fn sys_pow(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    match (a, b) {
        (NodeValue::I(av), NodeValue::I(bv)) => { Some(NodeValue::I(av.pow(*bv as u32))) },
        (NodeValue::F(av), NodeValue::I(bv)) => { Some(NodeValue::F(av.powi(*bv as i32))) },
        (_, _) => return Some(NodeValue::F(a.to_float(0.0).powf(b.to_float(1.0)))),
    }
}
fn sys_pow2(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    match a {
        NodeValue::I(av) => { Some(NodeValue::I(av.pow(2))) },
        NodeValue::F(av) => { Some(NodeValue::F(av.powi(2))) },
        _ => return Some(NodeValue::F(a.to_float(0.0).powi(2))),
    }
}
fn sys_gt(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_gt(a, b);
    Some(res)
}
fn sys_gteq(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_gteq(a, b);
    Some(res)
}
fn sys_lt(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_lt(a, b);
    Some(res)
}
fn sys_lteq(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_lteq(a, b);
    Some(res)
}
fn sys_eq(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let b = &args[1];
    let res = NodeValue::calc_eq(a, b);
    Some(res)
}
fn sys_typeof(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    let s = match a {
        NodeValue::B(_) => { "B" },
        NodeValue::I(_) => { "I" },
        NodeValue::F(_) => { "F" },
        NodeValue::S(_) => { "S" },
        _ => { "?" },
    };
    Some(NodeValue::from_str(s))
}
fn sys_toint(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    Some(NodeValue::I(a.to_int(0)))
}
fn sys_tofloat(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    Some(NodeValue::F(a.to_float(0.0)))
}
fn sys_hex(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    Some(NodeValue::S(format!("{:X}", a.to_int(0))))
}
fn sys_bin(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let a = &args[0];
    Some(NodeValue::S(format!("{:b}", a.to_int(0))))
}

#[cfg(test)]
mod test_runner {
    use super::super::runner::eval_str;

    #[test]
    fn test_calc() {
        let res = eval_str("3に5を足して表示");
        assert_eq!(res, "8");
        let res = eval_str("3を5倍して表示");
        assert_eq!(res, "15");
        let res = eval_str("3が5以上と表示");
        assert_eq!(res, "偽");
        let res = eval_str("3が5以下と表示");
        assert_eq!(res, "真");
        let res = eval_str("5が5以下と表示");
        assert_eq!(res, "真");
        let res = eval_str("5が5超と表示");
        assert_eq!(res, "偽");
        let res = eval_str("5が5と等しいと表示");
        assert_eq!(res, "真");
    }
    #[test]
    fn test_const() {
        let res = eval_str("カッコを表示");
        assert_eq!(res, "「");
    }
    #[test]
    fn test_typeof() {
        let res = eval_str("「あ」の変数型確認して表示");
        assert_eq!(res, "S");
        let res = eval_str("3の変数型確認して表示");
        assert_eq!(res, "I");
        let res = eval_str("3.0の変数型確認して表示");
        assert_eq!(res, "F");
        let res = eval_str("255のHEXを表示");
        assert_eq!(res, "FF");
        let res = eval_str("255の二進を表示");
        assert_eq!(res, "11111111");
    }
}
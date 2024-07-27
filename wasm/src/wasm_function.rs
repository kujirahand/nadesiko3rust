//! なでしこの標準関数を定義したもの

use wasm_bindgen::prelude::*;
use crate::node::*;

/// 関数をシステムに登録する
#[allow(dead_code)]
pub fn register(ctx: &mut NodeContext) {
    ctx.add_sysfunc("HOGE", sysargs(&[&["を", "と"]]), sys_hoge);
    ctx.add_sysfunc("言", sysargs(&[&["を", "と"]]), sys_say);
}

// `alert`関数をJavaScriptからインポート
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

fn sys_say(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let n = args[0].to_string();
    alert(&n);
    None
}

fn sys_hoge(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let n = args[0].to_string();
    Some(NodeValue::S(n))
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
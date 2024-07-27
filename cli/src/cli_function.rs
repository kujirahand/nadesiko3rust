//! なでしこの標準関数を定義したもの

use nadesiko3::node::*;
use std::{thread, time};

/// 関数をシステムに登録する
#[allow(dead_code)]
pub fn register(ctx: &mut NodeContext) {
    ctx.add_sysfunc("秒待", sysargs(&[&[""]]), fn_sleep);
    ctx.add_sysfunc("HOGE", sysargs(&[&["を"]]), fn_hoge);
}

fn fn_hoge(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let n = args[0].to_string();
    let n = format!("[HOGE: {}]", n);
    Some(NodeValue::from_str(&n))
}

fn fn_sleep(_: &mut NodeContext, args: Vec<NodeValue>) -> Option<NodeValue> {
    let n = args[0].to_float(0.0);
    let sec = time::Duration::from_secs_f64(n);
    thread::sleep(sec);
    None
}

#[cfg(test)]
mod test_runner {
    use super::*;
    use nadesiko3::{runner::*};

    #[test]
    fn test_calc() {
        let mut ctx = NodeContext::new();
        ctx.set_filename("test.nako3");
        nadesiko3::sys_function::register(&mut ctx); // register functions
        super::register(&mut ctx); // register functions
        //
        let result = eval_context(&mut ctx, "「ABC」を表示").unwrap();
        assert_eq!(result.to_string(), "ABC");
        //
        ctx.print_log.clear();
        let result = eval_context(&mut ctx, "「ABC」をHOGEして表示").unwrap();
        assert_eq!(result.to_string(), "[HOGE: ABC]");
    }
}

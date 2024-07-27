//! # Nadesiko Script v3
//! Nadesiko is one of Japanese programming languages.
//! You can write the code using Japanese.
//! 
//! # Example
//! ```
//! use nadesiko3rust::*;
//! // 文字を表示
//! let result = eval_str("「こんにちは」と表示");
//! println!("{}", result);
//! // 計算して表示
//! let result = eval_str("1+2×3と表示");
//! println!("{}", result);
//! // 以下のように記述することもできます
//! let result = eval_str("1に2を足して表示");
//! println!("{}", result);
//! ```
//! 
//! # Current Structure
//! ```text
//! source(&str) → Tokenize(Vec<Token>) → Parse(Vec<Node>) => Run(runner)
//! ```
//! @see runner::eval()
//! 

mod wasm_function;
use wasm_bindgen::prelude::*;

use nadesiko3::*;


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// モジュール呼び出し側で定義しなければいけない関数
#[wasm_bindgen]
extern "C" {
    fn nako3_handler(name: &str, s: &str) -> String;
}

/// 引数codeに指定したプログラムを実行して結果を文字列で返す
pub fn eval_str(code: &str) -> String {
    runner::eval_str(code)
}

/// 引数codeに指定したプログラムを実行して結果をNodeValueで返す
pub fn eval(code: &str) -> Result<node::NodeValue, String> {
    runner::eval(code, runner::RunOption::normal())
}

// #[cfg(target_arch = "wasm32")]
mod wasm {
    use node::*;
    use wasm_bindgen::prelude::*;
    use super::*;

    pub fn nako3_print(s: &str) {
        nako3_handler("print", s);
    }

    pub fn nako3_run(ctx: &mut NodeContext, code: &str) -> Result<NodeValue, String> {
        ctx.print_fn = Some(nako3_print);
        ctx.set_filename("main.nako3");
        nadesiko3::sys_function::register(ctx);
        wasm_function::register(ctx);
        runner::eval_context(ctx, code)
    }

    #[wasm_bindgen]
    pub fn nako_eval_str(code: &str) -> String {
        let mut ctx = NodeContext::new();
        let result = nako3_run(&mut ctx, code);
        match result {
            Ok(v) => v.to_string(),
            Err(e) => format!("!!{}", e.to_string()),
        }
    }
    #[wasm_bindgen]
    pub fn nako_eval_getlogs(code: &str) -> String {
        let mut ctx = NodeContext::new();
        let result = nako3_run(&mut ctx, code);
        match result {
            Ok(_) => ctx.print_log.clone(),
            Err(e) => format!("!!{}", e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eval_str_test() {
        let r = eval_str("「こんにちは」と表示。");
        assert_eq!(r, String::from("こんにちは"));
        let r = eval_str("1+2×3と表示。");
        assert_eq!(r, String::from("7"));
        let r = eval_str("1に2を足して3を掛けて表示。");
        assert_eq!(r, String::from("9"));
    }
}

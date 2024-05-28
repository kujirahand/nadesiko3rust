//! # Nadesiko Script v3
//! Nadesiko is one of Japanese programming languages.
//! You can write the code using Japanese.
//! 
//! # Example
//! ```
//! use nadesiko3::*;
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
//! # Build for Command line
//! ```sh
//! % cargo build
//! % nadesiko3 eval "1+2*3を表示"
//! ```
//! 
//! # Current Structure
//! source(&str) => Tokenize(Vec<Token>) => Parse(Vec<Node>) => Run
//! see runner::eval()
//! 

pub mod nvalue;
pub mod prepare;
pub mod strcur;
pub mod token;
pub mod kanautils;
pub mod tokenizer;
pub mod josi_list;
pub mod parser;
pub mod reserved_words;
pub mod tokencur;
pub mod node;
pub mod operator;
pub mod bytecode_gen;
pub mod bytecode_run;
// runner
pub mod runner;
pub mod sys_function;
pub mod sys_function_debug;

/// 引数codeに指定したプログラムを実行して結果を文字列で返す
pub fn eval_str(code: &str) -> String {
    runner::eval_str(code)   
}

/// 引数codeに指定したプログラムを実行して結果をNodeValueで返す
pub fn eval(code: &str) -> Result<node::NodeValue, String> {
    runner::eval(code, runner::RunOption::normal())
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
}

#[cfg(not(target_arch = "wasm32"))]
mod cli {
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

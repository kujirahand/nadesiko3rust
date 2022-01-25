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
pub mod runner;
pub mod operator;
pub mod sys_function;
pub mod sys_function_debug;

/// # Nadesiko3 script for Rust
/// Japanese programming language "Nadesiko"
/// - [URL] https://github.com/kujirahand/nadesiko3rust
/// 
/// # Example
/// ```
/// use nadesiko3::*;
/// // 計算して表示
/// let result = eval_str("1+2×3と表示");
/// println!("{}", result);
/// // 文字を表示
/// let result = eval_str("「こんにちは」と表示");
/// println!("{}", result);
/// // 日本語式
/// let result = eval_str("1に2を足して表示");
/// println!("{}", result);
/// ```

pub fn eval_str(code: &str) -> String {
    runner::eval_str(code)   
}

pub fn eval(code: &str) -> Result<node::NodeValue, String> {
    runner::eval(code, runner::RunOption::normal())
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

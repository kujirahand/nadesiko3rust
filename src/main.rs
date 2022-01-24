mod prepare;
mod strcur;
mod token;
mod kanautils;
mod tokenizer;
mod josi_list;
mod parser;
mod reserved_words;
mod tokencur;
mod node;
mod context;
mod runner;
mod operator;
mod sys_function;
mod sys_function_debug;
use std::fs;

fn main() {
    let mut src = String::from("");
    let mut filename: Option<String> = None;
    let mut debug_mode = false;
    let mut eval_mode = false;
    let mut runtime = String::from("");
    for (i, arg) in std::env::args().enumerate() {
        if i == 0 { runtime = arg; continue; } // 自分自身
        println!("{}:{}", i, arg);
        if arg.eq("") { continue; }
        let ch = arg.chars().nth(0).unwrap_or('\0');
        if ch == '-' { // option
            if arg.eq("-d") { debug_mode = true; }
            if arg.eq("-e") { eval_mode = true; }
            continue;
        }
        // [memo] cargo run でもevalモードが使えるように「-」なしのモード
        if arg.eq("e") || arg.eq("eval") { eval_mode = true; continue; }
        if eval_mode {
            src = arg;
            continue;
        }
        if filename == None {
            filename = Some(arg);
            continue;
        }
    }
    // 何も指定がなかったとき
    if filename == None && src.eq("") {
        show_usage(); return;
    }
    if debug_mode { println!("=== DEBUG mode({}) ===", runtime); }
    let filename = match filename {
        Some(fname) => {
            src = match fs::read_to_string(&fname) {
                Ok(s) => s,
                Err(err) => return println!("ソースファイル『{}』が読めません。{}", fname, err),
            };
            fname
        }
        None => String::from("eval"),
    };
    if eval_mode {
        if debug_mode { runner::eval_simple_str(&src); }
        else { runner::eval_str(&src); }
        return;
    }
    compile_and_run(&src, &filename);
}

fn compile_and_run(src: &str, fname: &str) {
    // prepare
    let mut parser = parser::Parser::new();
    sys_function::register(&mut parser.context);
    // tokenizer
    println!("--- tokenize ---");
    let tokens = tokenizer::tokenize(src);
    println!("{}", token::tokens_string(&tokens));
    println!("--- parse ---");
    let nodes = match parser.parse(tokens, fname) {
        Ok(nodes) => nodes,
        Err(e) => { println!("!!{}", e); return },
    };
    println!("{}", node::nodes_to_string(&nodes, "\n"));
    println!("--- run ---");
    match runner::run_nodes(&mut parser.context, &nodes) {
        Ok(v) => println!("{:?}", v),
        Err(e) => println!("!!{}", e),
    }
    println!("ok.");   
}

fn show_usage() {
    println!(
        "{}\n{}\n{}\n{}\n{}",
        "[nadesiko3rust]",
        "[使い方] > nadesiko3 (options) (filename)",
        "options:",
        "  -e, e, eval  ... ソースを直接指定して実行",
        "  -d           ... デバッグ情報を表示",
    );
}


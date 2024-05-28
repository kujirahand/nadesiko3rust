/// # Nadesiko3 script for Rust
/// Japanese programming language "Nadesiko"
/// - <https://github.com/kujirahand/nadesiko3rust>

use nadesiko3rust::{node::NodeContext, *};
use std::fs;

fn main() {
    let mut src = String::from("");
    let mut filename: Option<String> = None;
    let mut debug_mode = false;
    let mut eval_mode = false;
    let mut parse_mode = false;
    let mut runtime = String::from("");
    for (i, arg) in std::env::args().enumerate() {
        if i == 0 { runtime = arg; continue; } // 自分自身
        if arg.eq("") { continue; }
        let ch = arg.chars().nth(0).unwrap_or('\0');
        if ch == '-' { // option
            if arg.eq("-d") { debug_mode = true; }
            if arg.eq("-e") { eval_mode = true; }
            if arg.eq("-p") { parse_mode = true; }
            continue;
        }
        // [memo] cargo run でもevalモードが使えるように「-」なしのモード
        if arg.eq("e") || arg.eq("eval") { eval_mode = true; continue; }
        if arg.eq("d") || arg.eq("debug") { debug_mode = true; continue; }
        if arg.eq("p") || arg.eq("parse") { parse_mode = true; continue; }
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
    if parse_mode { debug_mode = true; }
    compile_and_run(&src, &filename, debug_mode, parse_mode);
}

fn include_file(context: &mut NodeContext, tokens: Vec<token::Token>) -> Vec<token::Token> {
    // 取り込むが使われている？
    let (mut tokens, files) = tokenizer::read_include_files(tokens);
    if files.len() > 0 {
        for f in files.iter() {
            // 既に読み込み済みであればスキップ
            if let Some(_no) = context.find_files(f) { continue; }
            // ファイルを読み込む
            // TODO: パスを探す処理
            let src = match fs::read_to_string(f) {
                Ok(s) => s,
                Err(err) => { println!("取り込み対象ファイル『{}』が読めません。{}", f, err); return vec![]; },
            };
            // トークンに変換
            let fileno = context.set_filename(f);
            let mut included_tokens = tokenizer::tokenize(&src, 0, fileno);
            included_tokens.extend(tokens);
            tokens = included_tokens;
        }
    }
    tokens
}

fn compile_and_run(src: &str, fname: &str, debug_mode: bool, parse_mode: bool) {
    // prepare
    let mut context = NodeContext::new();
    context.debug_mode = debug_mode;
    sys_function::register(&mut context);
    let fileno = context.set_filename(fname);

    // tokenizer
    if debug_mode { println!("--- tokenize ---"); }
    let tokens = tokenizer::tokenize(src, 0, fileno);
    if debug_mode { println!("{}", token::tokens_string(&tokens)); }

    if debug_mode { println!("--- include ---"); }
    let tokens = include_file(&mut context, tokens);

    // parser
    if debug_mode { println!("--- parse ---"); }
    let mut parser = parser::Parser::new_context(tokens, context);
    let nodes = match parser.parse() {
        Ok(nodes) => nodes,
        Err(e) => { println!("!!{}", e); return },
    };
    if debug_mode {
        println!("--- nodes ---");
        // println!("Nodes={:?}", nodes);
        println!("{}", node::nodes_to_string_lineno(&nodes, "\n"));
        println!("--- user function ---");
        // グローバルな関数をチェック
        let g_scope = &parser.context.scopes.scopes[1];
        for (_key, no) in g_scope.var_names.iter() {
            let v: &node::NodeValue = &g_scope.var_values[*no];
            match v {
                node::NodeValue::CallFunc(name, _no, nodes) => {
                    println!("●{}", name);
                    println!("{}", node::nodes_to_string_lineno(nodes, "\n"));
                },
                _ => {},
            }
        }
        println!("--- run ---"); 
    }
    if parse_mode { return; }
    /*
    // TODO: byte code
    let codes = match bytecode_gen::generate(&nodes) {
        Ok(codes) => codes,
        Err(e) => { println!("[ERROR] {}", e); return; },
    };
    println!("{:?}", codes);
    */
    // run_nodes
    context = parser.context;
    match runner::run_nodes(&mut context, &nodes) {
        Ok(v) => if debug_mode { println!(">> {}", v.to_string()); },
        Err(e) => println!("!!{}", e),
    }
}

fn show_usage() {
    println!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        "[nadesiko3rust]",
        "[使い方] > nadesiko3 (options) (filename)",
        "options:",
        "  -e, e, eval  ... ソースを直接指定して実行",
        "  -d, d, debug ... デバッグ情報を表示",
        "  -p, p, parse ... パースだけして表示",
    );
}


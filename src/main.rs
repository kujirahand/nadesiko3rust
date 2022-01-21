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
mod runner;
mod operator;
mod sys_function;
mod sys_function_debug;


fn main() {
    // let src = "a = 30; b=40; aをデバッグ表示;";
    // let src = "(2に3を足す)を表示;";
    let src = "「hoge」を表示";
    
    // prepare
    let mut parser = parser::Parser::new();
    sys_function::register(&mut parser.context);
    // tokenizer
    println!("--- tokenize ---");
    let tokens = tokenizer::tokenize(src);
    println!("{:?}", tokens);
    println!("--- parse ---");
    let nodes = match parser.parse(tokens, "a.nako3") {
        Ok(nodes) => nodes,
        Err(e) => { println!("!!{}", e); return },
    };
    println!("{}", node::nodes_to_string(&nodes, "\n"));
    println!("--- run ---");
    let v = runner::run_nodes(&mut parser.context, &nodes);
    println!("{:?} || {}", v, v.to_string());
    
    // ---------------
    println!("=== easy method ===");
    runner::eval_str("「気前よく与えてより豊かになる人がいる。」と表示。");
}


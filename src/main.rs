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


fn main() {
    // let src = "a = 30; b=40; aをデバッグ表示;";
    // let src = "(2に3を足す)を表示;";
    // let src = "「hoge」を表示";
    //let src = "もし、3=3ならば「OK」と表示。違えば「NG」と表示。";
    let src = "真&&真と表示";
    // prepare
    let mut parser = parser::Parser::new();
    sys_function::register(&mut parser.context);
    // tokenizer
    println!("--- tokenize ---");
    let tokens = tokenizer::tokenize(src);
    println!("{}", token::tokens_string(&tokens));
    println!("--- parse ---");
    let nodes = match parser.parse(tokens, "a.nako3") {
        Ok(nodes) => nodes,
        Err(e) => { println!("!!{}", e); return },
    };
    println!("{}", node::nodes_to_string(&nodes, "\n"));
    println!("--- run ---");
    match runner::run_nodes(&mut parser.context, &nodes) {
        Ok(v) => println!("{:?}", v),
        Err(e) => println!("!!{}", e),
    }
    
    // ---------------
    println!("=== easy method ===");
    runner::eval_str("「気前よく与えてより豊かになる人がいる。」と表示。");
    runner::eval_simple_str("「なでしこ」と表示。");
}


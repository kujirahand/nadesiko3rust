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


fn main() {
    // let tokens = tokenizer::tokenize("a = 30; b=40; aをデバッグ表示;");
    let tokens = tokenizer::tokenize("(2に3を足す)を表示;");
    let mut parser = parser::Parser::new();
    sys_function::register(&mut parser.context);
    parser.parse(tokens, "a.nako3");
    println!("--- parse ---");
    println!("{}", node::nodes_to_string(&parser.nodes, "\n"));
    let mut ctx = parser.clone_context();
    // println!("--- scopes ---\n{:?}----\n", ctx.scopes);
    println!("--- run ---");
    let v = runner::run_nodes(&mut ctx, &parser.nodes);
    println!("{:?} || {}", v, v.to_string());
}


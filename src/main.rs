mod prepare;
mod strcur;
mod kanautils;
mod tokenizer;
mod josi_list;
mod parser;
mod reserved_words;
mod tokencur;
mod node;
mod runner;

fn main() {
    // let tokens = tokenizer::tokenize("a = 30; b=40; aをデバッグ表示;");
    let tokens = tokenizer::tokenize("123をデバッグ表示;");
    let mut parser = parser::Parser::new();
    parser.parse(tokens, "hoge");
    println!("--- parse ---");
    println!("{}", node::nodes_to_string(&parser.nodes, "\n"));
    let mut ctx = parser.clone_context();
    println!("--- run ---");
    let v = runner::run_nodes(&mut ctx, &parser.nodes);
    println!("{:?} || {}", v, v.to_string());
}

mod prepare;
mod strcur;
mod charutils;
mod tokenizer;
mod josi_list;
mod parser;
mod reserved_words;
mod tokencur;
mod node;

fn main() {
    let tokens = tokenizer::tokenize("//test");
    let mut parser = parser::Parser::new(tokens, "hoge");
    parser.parse();
    println!("{:?}", parser.nodes);
}

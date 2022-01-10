mod prepare;
mod strcur;
mod charutils;
mod tokenizer;
mod josi_list;
mod parser;
mod reserved_words;

fn main() {
    println!("Hello, world!");
    let tokens = tokenizer::tokenize("//test");
    println!("{:?}", tokens);
}

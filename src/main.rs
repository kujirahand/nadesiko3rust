mod prepare;
mod strcur;
mod charutils;
mod tokenizer;

fn main() {
    println!("Hello, world!");
    let tokens = tokenizer::tokenize("//test");
    println!("{:?}", tokens);
}

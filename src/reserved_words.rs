//! 予約語を定義したもの
// @see https://github.com/kujirahand/nadesiko3/blob/master/src/nako_reserved_words.js

use crate::token::TokenType;

/// 文字列が予約語かどうか調べて TokenKind に変換
pub fn check_kind(s: &str) -> TokenType {
    // Word => 予約語
    if s == "もし" { return TokenType::If; }
    if s == "回" { return TokenType::Kai; }
    if s == "ここまで" { return TokenType::BlockEnd; }
    if s == "ここから" { return TokenType::BlockBegin; }
    if s == "かつ" { return TokenType::And; }
    if s == "または" { return TokenType::Or; }
    if s == "違" { return TokenType::Else; }
    if s == "抜" { return TokenType::Break; }
    if s == "続" { return TokenType::Continue; }
    if s == "繰返" { return TokenType::For; }
    if s == "戻" { return TokenType::Return; }
    if s == "変数" { return TokenType::DefVar; }
    if s == "代入" { return TokenType::Dainyu; }
    // todo
    TokenType::Word
}

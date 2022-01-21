// 予約語
// @see https://github.com/kujirahand/nadesiko3/blob/master/src/nako_reserved_words.js

use crate::token::TokenKind;

pub fn check_kind(s: &str) -> TokenKind {
    // Word => 予約語
    if s == "もし" { return TokenKind::If }
    if s == "回" { return TokenKind::Repeat }
    if s == "ここまで" { return TokenKind::BlockEnd }
    // todo
    TokenKind::Word
}
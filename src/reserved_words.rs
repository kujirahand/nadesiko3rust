//! 予約語を定義したもの
// @see https://github.com/kujirahand/nadesiko3/blob/master/src/nako_reserved_words.js

use crate::token::TokenKind;

/// 文字列が予約語かどうか調べて TokenKind に変換
pub fn check_kind(s: &str) -> TokenKind {
    // Word => 予約語
    if s == "もし" { return TokenKind::If; }
    if s == "回" { return TokenKind::Kai; }
    if s == "ここまで" { return TokenKind::BlockEnd; }
    if s == "ここから" { return TokenKind::BlockBegin; }
    if s == "かつ" { return TokenKind::And; }
    if s == "または" { return TokenKind::Or; }
    if s == "違" { return TokenKind::Else; }
    if s == "抜" { return TokenKind::Break; }
    if s == "続" { return TokenKind::Continue; }
    if s == "繰返" { return TokenKind::For; }
    if s == "戻" { return TokenKind::Return; }
    if s == "変数" { return TokenKind::DefVar; }
    if s == "代入" { return TokenKind::Dainyu; }
    // todo
    TokenKind::Word
}

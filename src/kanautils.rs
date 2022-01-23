/**
 * kanautils.rs
 * 半角変換変換用
 */


use std::char;

#[allow(dead_code)]
pub fn is_half(c: char) -> bool {
    (c as u32) < 0xF0u32
}

macro_rules! in_range {
    ( $v:expr => $( $a:expr ),* ) => {
        $( ($a).contains( & $v ) || )* false
    };
}

#[allow(dead_code)]
pub fn is_alpha(c: char) -> bool {
    in_range![c => 'a'..='z', 'A'..='Z']
}

#[allow(dead_code)]
pub fn is_numeric(c: char) -> bool {
    ('0'..='9').contains(&c)
}

pub fn is_hiragana(c: char) -> bool {
    // 3041-309F
    ('ぁ'..='ゟ').contains(&c)
    // 正確に言えば...
    // 'ぁ'..='ゔ', 'ゕ'..='ゖ', 'ゝ'..='ゞ', 'ゟ'..='ゟ',
}

pub fn is_word_chars(c: char) -> bool {
    let cu: u32 = c as u32;
    // ASCII領域
    if cu <= 0xFF {
        if in_range![
            c => '0'..='9', 'a'..='z', 'A'..='Z', '_'..='_'
        ] { return true; }
        return false;
    }
    // 非ASCII領域
    // @see https://www.asahi-net.or.jp/~ax2s-KMTN/ref/unicode/index_u.html
    /*
    // 日本語で使う仮名領域
    (0x3040 as char) ..= (0x309F as char), // ひらがな
    (0x30A0 as char) ..= (0x30FF as char), // カタカナ
    (0x1B000 as char) ..= (0x1B16F as char), // かな補助領域
    (0xFF00 as char) ..= (0xFFEF as char), // 半角カナ
    (0x3190 as char) ..= (0x319F as char), // 漢文用記号
    // 漢字領域
    (0x2F00 as char) ..= (0x31EF as char), // 部首字画など
    (0x3400 as char) ..= (0x9FFC as char), // CJK統合漢字+A
    (0xF900 as char) ..= (0xFAFF as char), // CJK互換漢字
    (0x20000 as char) ..= (0x3134A as char), // CJK統合漢字B-G
    (0xE0100 as char) ..= (0xE01EF as char), // 異体字セレクタ
    */
    // 基本OKだが全角記号などは変数名に使えない
    if in_range![
        cu => 
        0x2190..=0x21FF, // 矢印領域
        0x25A0..=0x25FF, // 幾何学模様(●や▲)
        0x3000..=0x303F  // CJKの記号と句読点(「」や【】や『』) @see https://www.asahi-net.or.jp/~ax2s-KMTN/ref/unicode/u3000.html
    ] { return false; }
    return true;
}

pub fn char_from_u32(i: u32, def: char) -> char {
    char::from_u32(i).unwrap_or(def)
}

// https://en.wikipedia.org/wiki/Halfwidth_and_Fullwidth_Forms_(Unicode_block)
pub fn to_half_ascii(c: char) -> char {
    let ci = c as u32;
    match ci {
        // half ascii code
        0x0020..=0x007E => c,
        // '！'..='～' = '\u{FF01}'..='\u{FF5E}'
        0xFF01..=0xFF5E => char_from_u32(ci - 0xFF01 + 0x21, c),
        // space
        0x2002..=0x200B => ' ',
        0x3000 | 0xFEFF => ' ',
        // others
        _ => c,
    }
}

#[cfg(test)]
mod test_charutils {
    use super::*;
    #[test]
    fn test_to_half() {
        assert_eq!(is_half('!'), true);
        assert_eq!(is_half('！'), false);
        assert_eq!('！' as u32, 0xFF01);
        assert_eq!(to_half_ascii('！'), '!');
        assert_eq!(to_half_ascii('Ａ'), 'A');
        assert_eq!(to_half_ascii('＃'), '#');
        assert_eq!(to_half_ascii('　'), ' ');
    }
    #[test]
    fn test_range() {
        assert_eq!(is_alpha('a'), true);
        assert_eq!(is_alpha('B'), true);
        assert_eq!(is_alpha('3'), false);
        assert_eq!(is_alpha('$'), false);
    }
}
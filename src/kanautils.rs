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
}

pub fn is_word_chars(c: char) -> bool {
    if in_range![c => 'a'..='z', 'A'..='Z', '_'..='_', '0'..='9'] { return true; }
    if (c as u32) >= 0xE0 { return true; }
    return false;
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
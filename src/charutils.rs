/**
 * charutils.rs
 * 半角変換変換用
 */

use std::char;
pub fn is_half(c: char) -> bool {
    (c as u32) < 0xF0u32
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
}
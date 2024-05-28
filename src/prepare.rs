//! 前置処理

use crate::kanautils;
use crate::strcur::StrCur;

/// 全角半角、記号などを揃える
pub fn convert(src: &str, fileno: i32) -> String {
    let mut result = String::new();
    let mut cur = StrCur::from(src, fileno);
    let mut is_str = false;
    let mut end_of_str = 0 as char;
    let mut is_line_comment = false;
    let mut is_range_comment = false;
    while cur.can_read() {
        let ch = cur.next();
        let ch_half = kanautils::to_half_ascii(ch);
        // string
        if is_str {
            if ch == end_of_str {
                is_str = false;
                result.push(ch);
                continue;
            }
            result.push(ch);
            continue;
        }
        // comment
        if is_line_comment {
            if ch == '\n' {
                is_line_comment = false;
            }
            result.push(ch);
            continue;
        }
        if is_range_comment {
            if ch_half == '*' {
                let ch2 = cur.peek_half();
                if ch2 == '/' {
                    is_range_comment = false;
                    result.push_str("*/");
                    cur.next();
                    continue;
                }
            }
            result.push(ch);
            continue;
        }

        // check string
        if ch == '"' || ch == '\'' {
            is_str = true;
            end_of_str = ch;
            result.push(ch);
            continue;
        }
        if ch == '「' {
            is_str = true;
            end_of_str = '」';
            result.push(ch);
            continue;
        }
        if ch == '『' {
            is_str = true;
            end_of_str = '』';
            result.push(ch);
            continue;
        }
        // check comment
        if ch_half == '/' {
            let ch2 = cur.peek_half();
            if ch2 == '/' {
                is_line_comment = true;
                result.push_str("//");
                cur.next();
                continue;
            }
            if ch2 == '*' {
                is_range_comment = true;
                result.push_str("/*");
                cur.next();
                continue;
            }
        }
        // その他の変換
        let cc = match ch_half {
            '。' => ';', // 語尾
            '、' => ',',
            _ => ch_half,
        };
        result.push(cc);
    }
    return result;
}

#[cfg(test)]
mod test_prepare {
    use super::*;
    #[test]
    fn convert_test() {
        let s = convert("", 0);
        assert_eq!(s, String::from(""));
        let s = convert("abc", 0);
        assert_eq!(s, String::from("abc"));
        let s = convert("！！/*！！*/！！", 0);
        assert_eq!(s, String::from("!!/*！！*/!!"));
        let s = convert("！！「！！」！！", 0);
        assert_eq!(s, String::from("!!「！！」!!"));
        let s = convert("！！『！！』！！", 0);
        assert_eq!(s, String::from("!!『！！』!!"));
        let s = convert("ＡＢＣ", 0);
        assert_eq!(s, String::from("ABC"));
    }
}

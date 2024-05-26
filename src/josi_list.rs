//! 助詞一覧を定義したもの

use crate::strcur::StrCur;

/// 助詞の一覧を定義
pub const JOSI_LIST: [&str; 46] = [
  // 参考 <https://github.com/kujirahand/nadesiko3/blob/master/src/nako_josi_list.js>
  // もし文で使う[助詞]
  "でなければ", "なければ", "ならば", "なら", "たら", "れば",
  // 一般的な助詞
  "について", "くらい", "なのか", "までを", "までの", 
  "による", "とは", "から", "まで", "だけ", 
  "より", "ほど", "など", "いて", "えて", 
  "きて", "けて", "して", "って", "にて", 
  "みて", "めて", "ねて", "では", "には", 
  "は~", "んで", "は", "を", "に", 
  "へ", "で", "と", "が", "の",
  // 意味のない語尾
  "こと", "である", "です", "します", "でした",
];

/// 意味のない語尾
const JOSI_LIST_IMINASI: [&str; 5] = [
  "こと", "である", "です", "します", "でした"
];

/// 「もし」文で使う助詞
const JOSI_LIST_MOSI: [&str; 4] = ["ならば", "なら", "たら", "れば"];
const JOSI_LIST_MOSI_NOT: [&str; 2] = ["でなければ", "なければ"];


/// 助詞を返す
pub fn read_josi(cur: &mut StrCur) -> Option<String> {
    for josi in JOSI_LIST {
      if cur.eq_str(josi) {
        // 助詞を見つけたらカーソルを進める
        cur.seek(josi.chars().count() as i64);
        // ただし、意味なしの助詞であればカーソルを進めつつもNoneを返す
        if is_josi_iminasi(josi) {
          return None;
        }
        // 助詞を返す
        return Some(String::from(josi));
      }
    }
    None
}

/// 「もし」文で使う助詞かどうか(助詞があればSome/肯定ならtrue,否定ならfalse)
pub fn is_josi_mosi(josi: &str) -> Option<bool> {
  // 肯定助詞
  for w in JOSI_LIST_MOSI {
    if w == josi { return Some(true) }
  }
  // 否定助詞
  for w in JOSI_LIST_MOSI_NOT {
    if w == josi { return Some(false) }
  }
  None
}

/// 意味のない助詞(語尾)かどうか (別の箇所で判定後削除する)
pub fn is_josi_iminasi(josi: &str) -> bool {
  for w in JOSI_LIST_IMINASI {
    if w == josi { return true }
  }
  false
}

#[cfg(test)]
mod test_josi {
    use super::*;
    #[test]
    fn is_josi_test() {
        // 文字はUTF-8の
        let mut cur = StrCur::from("について");
        assert_eq!(read_josi(&mut cur), Some(String::from("について")));
        //
        let mut cur = StrCur::from("Aでなければ");
        assert_eq!(read_josi(&mut cur), None);
        cur.next(); // skip A
        assert_eq!(read_josi(&mut cur), Some(String::from("でなければ")));
    }

    #[test]
    fn is_josi_mosi_test() {
        let s = String::from("でなければ");
        assert_eq!(is_josi_mosi(&s), Some(false));
    }
    
    #[test]
    fn is_josi_iminasi_test() {
      let s = String::from("です");
      assert_eq!(is_josi_iminasi(&s), true);
  }

}
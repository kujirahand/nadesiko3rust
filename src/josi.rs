// 助詞を定義したもの
// @see https://github.com/kujirahand/nadesiko3/blob/master/src/nako_josi_list.js
use crate::strcur::StrCur;

// 助詞のバイト数(文字数ではない)を調べる、助詞がなければ0を返す
pub fn is_josi(cur: &StrCur) -> usize {
    let josi_list = [
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
    for josi in josi_list {
      if cur.eq_str(josi) {
        return josi.len();
      }
    }
    0
}

// 「もし」文で使う助詞かどうか
pub fn is_josi_mosi(josi: &str) -> bool {
  let josi_list = [
      "でなければ", "なければ", "ならば", "なら", "たら", "れば"
  ];
  for w in josi_list {
    if w == josi { return true }
  }
  false
}

// 意味のない助詞かどうか(削除する)
pub fn is_josi_iminasi(josi: &str) -> bool {
  let josi_list = [
      "こと", "である", "です", "します", "でした"
  ];
  for w in josi_list {
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
        let cur = StrCur::from("について");
        assert_eq!(is_josi(&cur), 4 * 3);
        //
        let mut cur = StrCur::from("Aでなければ");
        cur.next(); // skip A
        assert_eq!(is_josi(&cur), 5 * 3);
    }

    fn is_josi_mosi_test() {
        let s = String::from("でなければ");
        assert_eq!(is_josi_mosi(&s), true);
    }
    fn is_josi_iminasi_test() {
      let s = String::from("です");
      assert_eq!(is_josi_iminasi(&s), true);
  }

}
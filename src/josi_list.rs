// 助詞を定義したもの
// @see https://github.com/kujirahand/nadesiko3/blob/master/src/nako_josi_list.js
use crate::strcur::StrCur;

// 助詞を返す、助詞でなければ0を返す
pub fn read_josi(cur: &mut StrCur) -> Option<String> {
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
        // 助詞を見つけたらカーソルを進める
        cur.seek(josi.chars().count() as i32);
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
        assert_eq!(is_josi_mosi(&s), true);
    }
    
    #[test]
    fn is_josi_iminasi_test() {
      let s = String::from("です");
      assert_eq!(is_josi_iminasi(&s), true);
  }

}
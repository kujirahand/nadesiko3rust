// 助詞を定義したもの
use crate::strcur::StrCur;

const josi_list:[&str; 35] = [
    "について", "くらい", "なのか", "までを", "までの", 
    "による", "とは", "から", "まで", "だけ", 
    "より", "ほど", "など", "いて", "えて", 
    "きて", "けて", "して", "って", "にて", 
    "みて", "めて", "ねて", "では", "には", 
    "は~", "んで", "は", "を", "に", 
    "へ", "で", "と", "が", "の"
];

// 助詞の文字数を調べる、助詞がなければ0を返す
pub fn is_josi(cur:StrCur) -> usize {
    for josi in josi_list {
        if cur.eq_str(josi) {
            return josi.len();
        }
    }
    0
}

/*  
  // 「もし」文で使う助詞
  const tararebaJosiList = [
    "でなければ", "なければ", "ならば", "なら", "たら", "れば"
  ]
  
  // 意味のない助詞(削除する)
  const removeJosiList = [
    "こと", "である", "です", "します", "でした"
  ]
*/

#[cfg(test)]
mod test_josi {
    use super::*;
    #[test]
    fn is_josi_test() {
        let cur = StrCur::from("ならば");
        assert_eq!(is_josi(cur), 3);
    }
}
# 日本語プログラミング言語「なでしこv3」(Rust実装版)

このプロジェクトは、日本語プログラミング言語「なでしこ3」をプログラミング言語Rustで差異実装するプロジェクトです。
オリジナルの「なでしこ3」はJavaScriptで実装されています。

- [日本語プログラミング言語「なでしこ3」のリポジトリ](https://github.com/kujirahand/nadesiko3)

## インストールして利用する方法

Rust/Cargoがインストールされていれば以下のコマンドで最新版をインストールできます。

```sh
cargo install nadesiko3
```

## リポジトリからコンパイルする方法

GitHubのリポジトリを取得してコンパイルして実行するには以下のコマンドを実行します。

```sh
git clone https://github.com/kujirahand/nadesiko3rust.git
cd nadesiko3rust
cargo run eval "「こんにちは」と表示"
```

## Rustのプロジェクトになでしこ3を組み込んで使う方法

最初に、`cargo init`を実行してプロジェクトを初期化します。
続いて、`Cargo.toml`の[dependencies]に`nadesiko3`を追記してください。

```
[dependencies]
nadesiko3 = "0.1" # ←これを追加
```

そして、`main.rs`に以下のようなコードを記述します。

```
use nadesiko3::*;

fn main() {
    // 文字を表示
    let result = eval_str("「こんにちは」と表示");
    println!("{}", result);
    // 計算して表示
    let result = eval_str("1+2×3と表示");
    println!("{}", result);
    // 以下のように記述することもできます
    let result = eval_str("1に2を足して表示");
    println!("{}", result);
}
```

## 実装状況

条件分岐の「もし」文や繰り返しの「繰り返し」、関数定義など、簡単な計算ができるようになりました。
なでしこv1/v3のプロジェクトを主に開発しているので、隙間時間に少しずつ開発します。

フィボナッチやFizzBuzzのコードが動くくらいです。

```
# フィボナッチ
●FIB(Nの)
　　もし、N<2ならばNで戻る。
　　((N-1)のFIB)+((N-2)のFIB)で戻る。
ここまで。
(25のFIB)を表示。
```

```
# FizzBuzz
Nを1から100まで繰り返す
　　もし、(N%3=0)かつ(N%5=0)ならば「FizzBuzz」と表示。
　　違えば、もし、N%3=0ならば「Fizz」と表示。
　　違えば、もし、N%5=0ならば「Buzz」と表示。
　　違えば、Nを表示。
ここまで。
```

### 書き換え中

現在、バイトコードインタプリタを導入すべく修正中
動かないコードがあるかも。。。

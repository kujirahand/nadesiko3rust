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
cd nadesiko3rust/cli
cargo run eval "「こんにちは」と表示"
```

## Rustのプロジェクトになでしこ3を組み込んで使う方法

最初に、`cargo init`を実行してプロジェクトを初期化します。
続いて、`cargo add nadesiko3`を実行します。

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


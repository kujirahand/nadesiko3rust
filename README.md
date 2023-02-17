# 日本語プログラミング言語「なでしこv3」(Rust実装版)

このプロジェクトは、なでしこ3をプログラミング言語Rustで差異実装するプロジェクトです。
オリジナルの「なでしこ3」はJavaScriptで実装されています。

## コンパイルの方法

- (1) Rustをインストールします。
- (2) 以下のコマンドを実行します。
  - `cargo run`

## Rustから使う方法

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
配列がまだ実装できていません。

なでしこv1/v3のプロジェクトを主に開発しているので、隙間時間に少しずつ開発します。




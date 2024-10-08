# 日本語プログラミング言語「なでしこv3」(Rust実装版)

This project is Japanese Programming Language "Nadesiko3" for Rust.
Original was developped with JavaScript/TypeScript.

このプロジェクトは、日本語プログラミング言語「なでしこ3」をプログラミング言語Rustで再実装するプロジェクトです。
オリジナルの「なでしこ3」はJavaScriptで実装されています。

- [日本語プログラミング言語「なでしこ3」のリポジトリ(Oiriginal)](https://github.com/kujirahand/nadesiko3)


## 言語コアとWASM版

オリジナルの「なでしこ3」と区別するため、本リポジトリを`nadesiko3rust`としています。
また、WASM版のパッケージ名も、`nadesiko3rust`としています。

- [言語コア](core/README.md) ... 文法を定義したもの
- [コマンドライン版](cli/README.md) ... 言語コアを利用しつつコマンドラインに特化したもの
- [WASM版](wasm/README.md) ... 言語コアを利用しつつブラウザで実行できるようにしたもの


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

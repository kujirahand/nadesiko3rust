# 日本語プログラミング言語「なでしこv3」(Rust実装版)

このプロジェクトは、日本語プログラミング言語「なでしこ3」をプログラミング言語Rustで差異実装するプロジェクトです。
オリジナルの「なでしこ3」はJavaScriptで実装されています。

- [日本語プログラミング言語「なでしこ3」のリポジトリ](https://github.com/kujirahand/nadesiko3)

## 言語コアとWASM版のクレート

WASM版では、言語コアを参照しています。

- `nadesiko3` ... なでしこ3の言語コアのクレート
- `nadesiko3rust` ... WASM版のクレート

## WASM版のビルド方法

```sh
# wasm-packのインストール
cargo install wasm-pack
cd wasm
wasm-pack build --target web
```


## なでしこ3(RUST版)を利用する方法

下記のようなHTMLを作成します。

```html
<!DOCTYPE html><html lang="ja"><head><meta charset="UTF-8">
    <title>なでしこ3Rustテスト</title>
    <script type="module">
        import init, { nako_eval_str, nako_eval_getlogs } from './pkg/nadesiko3rust.js';
        async function runWasm() {
            await init(); // WASM モジュールを初期化
            // Rustで定義された関数を呼び出す
            const greeting = nako_eval_getlogs("「こんにちは、なでしこ3です。」と表示。");
            console.log(greeting); // コンソールに出力
        }
        runWasm().catch(console.error);
        // Rustから送出される表示ログなどを処理するハンドラ(必須)
        window.nako3_handler = (name, arg) => {
            console.log(name, arg);
            return arg;
        }
    </script>
</head>

<body>
</body>
</html>
```

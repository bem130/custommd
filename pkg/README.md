# Markdown Custom by Bem130
Rust製のMarkdown→HTML変換器＋WASM Playground
tags:
  - markdown
  - rust

## 概要

- Rust + WASMでMarkdownをHTMLに変換
- 独自のフロントマター（タイトル・説明・タグ）抽出
- セクションごとにネストしたHTML構造を生成
- AskamaテンプレートでOGP対応HTMLを生成
- Monaco EditorによるWeb上Markdownエディタ
- KaTeX/Highlight.jsによる数式・シンタックスハイライト対応
- タグバッジやダウンロード機能付き

## デモ

[`Demo`](https://bem130.github.io/custommd/) をブラウザで開くと、WASMでMarkdown→HTML変換が体験できます。

## 使い方

### Webで使う

1. [`index.html`](https://bem130.github.io/custommd/) を開く
2. 左ペインでMarkdownを編集
3. 右ペインでHTMLプレビュー・HTMLコードを確認
4. 「Markdownダウンロード」「HTMLダウンロード」ボタンで保存可能

### Rust CLIで使う

```sh
cargo run --release
```

- `src/main.rs` が `README.md` をHTMLに変換し、`output.html` を生成します。

## Markdown拡張仕様

- 1行目: タイトル
- 2行目以降: 説明文（空行または`tags:`まで）
- `tags:` 以降: `- タグ名` 形式でタグ列挙
- 以降: 本文Markdown

例:

```
# サンプルタイトル
これは説明文です。

tags:
- rust
- markdown

## セクション1

本文...
```

## 技術スタック

- Rust, wasm-bindgen, pulldown-cmark, askama, serde
- Web: Monaco Editor, KaTeX, Highlight.js

## ディレクトリ構成

```
src/
  lib.rs         // 変換ロジック
  main.rs        // CLI用
  style.css      // スタイル
templates/
  template.html  // Askamaテンプレート
index.html       // Playground本体
pkg/             // wasm-packビルド成果物
```

## ビルド方法

### WASMビルド

```sh
wasm-pack build --target web --out-dir pkg
```

### Webで動作確認

```sh
# 任意のHTTPサーバで index.html を開く
python -m http.server
# または
npx serve .
```

### CLIビルド

```sh
cargo run --release
```

## ライセンス

MIT

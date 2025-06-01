# Custom Markdown Playground
Rust + WASM で動作するカスタムMarkdown変換ツールです。

tags:
- rust
- markdown

---

Rust + WASM で動作するカスタムMarkdown変換ツールです。

## 概要

- RustでMarkdownをHTMLに変換し、WASMでブラウザ上で動作します。
- 独自のフロントマター（タイトル・説明・タグ）をサポート。
- セクションごとにHTMLをネストして出力。
- Monaco EditorでMarkdownとHTMLを編集・閲覧可能。

## 特徴

- `# タイトル`、説明文、`tags:`リスト、`---`区切り、本文 という独自フォーマットのMarkdownを解析。
- タグは自動的にリンク化されます。
- セクション見出し（h1〜h6）ごとにHTMLをネストし、可読性の高い構造を生成。
- RustのAskamaテンプレートでOGPタグやメタ情報も埋め込み可能。
- WASMビルドでWebブラウザ上で高速に動作。

## 使い方

1. `index.html` をブラウザで開く。
2. 左ペインでMarkdownを編集し、「変換」ボタンを押す。
3. 中央ペインでHTMLレンダリング結果、右ペインでHTMLコードを確認。

## Markdownフォーマット例

```md
# サンプルタイトル
説明文

tags:
- rust
- wasm

---

## セクション1
本文1

## セクション2
本文2
```

## ビルド方法

1. Rustとwasm-packをインストール。
2. `wasm-pack build --target web` でビルド。
3. `pkg/`配下のJS/WASMを`index.html`から読み込む。

## ディレクトリ構成

- `src/` ... Rustソース
- `templates/` ... Askamaテンプレート
- `pkg/` ... wasm-packビルド成果物
- `index.html` ... Web UI

## ライセンス

MIT License

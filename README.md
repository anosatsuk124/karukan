<div align="center">
  <img src="icon.png" width="128" alt="karukan" />
  <h1>Karukan</h1>
  <p>日本語入力システム — ニューラルかな漢字変換エンジン + fcitx5 / Windows TSF / macOS IMKit</p>

  [![CI (engine)](https://github.com/togatoga/karukan/actions/workflows/karukan-engine-ci.yml/badge.svg)](https://github.com/togatoga/karukan/actions/workflows/karukan-engine-ci.yml)
  [![CI (im)](https://github.com/togatoga/karukan/actions/workflows/karukan-im-ci.yml/badge.svg)](https://github.com/togatoga/karukan/actions/workflows/karukan-im-ci.yml)
  [![CI (macos)](https://github.com/togatoga/karukan/actions/workflows/karukan-macos-ci.yml/badge.svg)](https://github.com/togatoga/karukan/actions/workflows/karukan-macos-ci.yml)
  [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
</div>

<div align="center">
  <img src="images/demo.gif" width="800" alt="karukan demo" />
</div>

## プロジェクト構成

| クレート | 説明 |
|---------|------|
| [karukan-im](karukan-im/) | karukan-engineを利用したfcitx5向け日本語入力システム (Linux) |
| [karukan-tsf](karukan-tsf/) | karukan-engineを利用したTSF (Text Services Framework) 向け日本語入力システム (Windows) |
| [karukan-macos](karukan-macos/) | karukan-engineを利用したIMKit向け日本語入力システム (macOS) |
| [karukan-engine](karukan-engine/) | コアライブラリ — ローマ字→ひらがな変換 + llama.cppによるニューラルかな漢字変換 |
| [karukan-cli](karukan-cli/) | CLIツール・サーバー — 辞書ビルド、Sudachi辞書生成、辞書ビューア、AJIMEE-Bench、HTTPサーバー |

## フォークについて

このプロジェクトは [togatoga/karukan](https://github.com/togatoga/karukan) のフォークです。SKKモードの追加、Windows (TSF) およびmacOS (IMKit) への対応を行っています。

オリジナルの karukan を開発された [@togatoga](https://github.com/togatoga) さんに深く感謝いたします。

## 特徴

- **ニューラルかな漢字変換**: GPT-2ベースのモデルをllama.cppで推論し、高度な日本語変換
- **コンテキスト対応**: 周辺テキストを考慮した日本語変換
- **変換学習**: ユーザーが選択した変換結果を記憶し、次回以降の変換で優先表示。予測変換（前方一致）にも対応し、入力途中でも学習済みの候補を提示
- **システム辞書**: [SudachiDict](https://github.com/WorksApplications/SudachiDict)の辞書データからシステム辞書を構築
- **SKK辞書対応**: SKK-JISYO形式の辞書ファイルをユーザー辞書として読み込み可能（ファイル配置のみで自動検出）
- **abbrevモード**: `/` キーでraw input（abbrev）モードに入り、英字のまま辞書検索・変換（SKKキーバインド有効時）

> **Note:** 初回起動時にHugging Faceからモデルをダウンロードするため、初回の変換開始までに時間がかかります。2回目以降はダウンロード済みのモデルが使用されます。

## インストール

- **Linux (fcitx5)**: [karukan-im の README](karukan-im/README.md#install) を参照
- **Windows (TSF)**: [karukan-tsf の README](karukan-tsf/README.md#build) を参照
- **macOS (IMKit)**: [karukan-macos の README](karukan-macos/README.md) を参照

## ライセンス

MIT OR Apache-2.0 のデュアルライセンスで提供しています。

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

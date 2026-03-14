# karukan-macos

macOS IMKit 向け日本語入力メソッド

## ビルド

```bash
cd karukan-macos
scripts/build.sh
```

ビルド成果物は `karukan-macos/build/Karukan.app` に生成されます。

## インストール

```bash
scripts/install.sh
```

`~/Library/Input Methods/Karukan.app` にコピーされます。

> **重要:** macOS は `~/Library/Input Methods/` と `/Library/Input Methods/` のみを入力メソッドとして認識します。`/Applications/` に配置しても動作しません。

## 有効化

1. ログアウトしてログインし直す（または再起動）
2. System Settings > Keyboard > Input Sources > Edit > +
3. Japanese の下にある「Karukan」を追加

ログアウトせずに試す場合:

```bash
killall KarukanInputMethod 2>/dev/null
open ~/Library/Input\ Methods/Karukan.app
```

## 設定ファイル

設定ファイルのパスはプラットフォームにより異なります:

| プラットフォーム | パス |
|---|---|
| **macOS** | `~/Library/Application Support/com.karukan.karukan-im/config.toml` |
| **Linux** | `~/.config/karukan-im/config.toml` |

設定ファイルが存在しない場合はデフォルト値が使用されます。部分的な設定も可能です（未指定の項目はデフォルト値が適用されます）。

### 設定例

```toml
[conversion]
# 変換ストラテジー: adaptive, light, main
strategy = "adaptive"
# 変換候補数
num_candidates = 9
# 推論スレッド数（0 = 全コア）
n_threads = 4

[keybinding]
# キーバインドプロファイル: default, skk
profile = "skk"

[learning]
# 変換学習を有効にする
enabled = true
# 学習エントリの最大数
max_entries = 10000
```

### データディレクトリ

| 種類 | macOS | Linux |
|---|---|---|
| 学習キャッシュ | `~/Library/Application Support/com.karukan.karukan-im/learning.tsv` | `~/.local/share/karukan-im/learning.tsv` |
| ユーザー辞書 | `~/Library/Application Support/com.karukan.karukan-im/user_dicts/` | `~/.local/share/karukan-im/user_dicts/` |

## 既知の問題と回避策

### WezTerm で SKK モードの Ctrl+J が効かない

macOS 版の WezTerm では `Ctrl+J` が IME に渡されないため、SKK モード（`[keybinding] profile = "skk"`）でのひらがなモード切り替えができません。

[Karabiner-Elements](https://karabiner-elements.pqrs.org/) を使い、WezTerm 上でのみ `Ctrl+J` を「かな」キーにリマップすることで回避できます。「かな」キーは `Ctrl+J` と同じくひらがなモードへの切り替えとして動作します。

`~/.config/karabiner/assets/complex_modifications/` に以下の JSON ファイルを配置し、Karabiner-Elements の設定画面から有効化してください。

```json
{
  "title": "Karukan SKK: Ctrl+J to Kana key (WezTerm only)",
  "rules": [
    {
      "description": "Remap Ctrl+J to Kana key in WezTerm for Karukan SKK mode",
      "manipulators": [
        {
          "type": "basic",
          "from": {
            "key_code": "j",
            "modifiers": {
              "mandatory": ["control"]
            }
          },
          "to": [
            {
              "key_code": "japanese_kana"
            }
          ],
          "conditions": [
            {
              "type": "frontmost_application_if",
              "bundle_identifiers": [
                "^com\\.github\\.wez\\.wezterm$"
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

## 必要環境

- macOS 14.0 以上
- Rust toolchain
- Xcode Command Line Tools（swiftc が必要）

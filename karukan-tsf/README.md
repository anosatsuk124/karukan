# karukan-tsf

Windows向け日本語IME。TSF (Text Services Framework) 上で動作し、karukan-engineによるニューラルかな漢字変換を行います。

## Build

### Prerequisites

- Rust stable toolchain
- Windows 10/11 (MSVC ターゲット)

### Native build (Windows MSVC)

llama.cpp (CMake) がデフォルトで動的CRT (`/MD`) を使用するため、Rustのデフォルト静的CRT (`/MT`) と一致させる必要があります:

```powershell
set CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreaded
cargo build -p karukan-tsf --release
```

### Cross-compile from Linux

```bash
# mingw-w64 toolchain が必要
cargo build -p karukan-tsf --release --target x86_64-pc-windows-gnu
```

### Registration

ビルド後、管理者権限のコマンドプロンプトでDLLを登録します:

```powershell
regsvr32 target\release\karukan_tsf.dll
```

登録解除:

```powershell
regsvr32 /u target\release\karukan_tsf.dll
```

## Usage

### IME の ON/OFF 切り替え

| キー | 動作 |
|------|------|
| 半角/全角 | IME の ON/OFF トグル |
| Ctrl+Space | IME の ON/OFF トグル（半角/全角キーがないキーボード向け） |

### 入力モード切り替え

| キー | 動作 |
|------|------|
| Ctrl+K | カタカナモード（Composing 中） |
| Ctrl+Shift+L | ライブ変換の ON/OFF |

## Configuration

設定ファイル: `%APPDATA%\karukan\karukan-im\config\config.toml`

（例: `C:\Users\<ユーザー名>\AppData\Roaming\karukan\karukan-im\config\config.toml`）

設定ファイルが存在しない場合、デフォルト値が使用されます。部分的な設定も可能です（指定した項目のみ上書き）。

### SKK 風キーバインド

SKK 風のモード切り替えキーバインドを使用できます:

```toml
[keybinding]
profile = "skk"
```

| キー | 動作 |
|------|------|
| Ctrl+j | ひらがなモードに切り替え（どのモードからでも） |
| l | アルファベットモードに切り替え（かなモード時） |
| q | ひらがな ↔ カタカナをトグル（かなモード時） |
| Ctrl+q | 半角カタカナモードに切り替え |

### 設定例

```toml
[conversion]
strategy = "adaptive"       # adaptive / light / main
num_candidates = 9          # 変換候補数
n_threads = 4               # 推論スレッド数（0 = 全コア）

[keybinding]
profile = "default"         # default / skk

[learning]
enabled = true              # 変換学習の有効/無効
max_entries = 10000         # 学習エントリの最大数
```

## License

MIT OR Apache-2.0

# daily-akari-solver

Daily Akari (https://dailyakari.com) の盤面を補助するためのツール群です。
Chrome 拡張から盤面画像を取得し、ローカルの Go サーバでセル位置を検出します。
Rust 実装のソルバーや API なども同梱しています。

## 構成

- `main.go` / `detect/`: 盤面画像からセル位置を推定するローカル API
- `chrome_extension/`: Daily Akari で動作する Chrome 拡張
- `solver/akari/`: Akari のソルバー (Rust)
- `solver/api/`: ソルバー API (Rust / Wrangler)

## 使い方 (ローカル)

1. 依存関係の準備
   - Go (go.mod を参照)
   - OpenCV + gocv (https://gocv.io/getting-started/)
2. ローカル API を起動

```bash
go run ./main.go
```

3. Chrome 拡張を読み込む
   - `chrome://extensions` を開き、デベロッパーモードを ON
   - 「パッケージ化されていない拡張機能を読み込む」から `chrome_extension/` を選択
4. https://dailyakari.com を開き、ページ上部の「Solve Akari」ボタンをクリック

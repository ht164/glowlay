# glowlay

`glowlay` は、Linuxデスクトップ環境（Wayland / X11）で動作する、軽量かつ美麗なデスクトップリソースモニター（オーバーレイ）です。  
RustとGTK4、および gtk4-layer-shell を用いて構築されており、システムに余分な負荷をかけることなく常駐動作します。

## 主な機能
* **超軽量設計**: Rustによる実装で、常駐時のCPU/メモリオーバーヘッドを極小に抑えます。
* **背景・クリック透過**: ウィジェットの背景が綺麗に透過し、ウィジェット上のマウスクリックは背後のウィンドウやデスクトップへすり抜けます。
* **ネオングロー効果**: 青白い発光を伴う美しいダークテーマUI。
* **複数ディスプレイサーバー対応**: Wayland環境（Sway, Hyprland, GNOME, KDE等）とX11環境の双方で自動的にオーバーレイ配置が行われます。
* **リソース監視**:
  * **RAM & VRAM**: 円形メーターによる使用率表示（GPUはNVIDIA製に対応）
  * **CPU**: スプライン曲線による滑らかな使用率履歴グラフ
  * **TEMP**: CPU、システム、および GPU（NVIDIA）温度の横バー表示
  * **STORAGE**: 指定した複数のマウントポイント（ルート `/` や `/home` など）の使用状況を `使用量 (使用率)` のコンパクトな形式で一覧表示（Grid整列・固定幅）
  * **NETWORK**: 送受信速度（DOWN / UP）のリアルタイム表示（Grid整列・固定幅・等幅フォント）

---

## 導入およびビルド方法

### 1. 依存関係のインストール (Gentoo Linux の例)
GTK4 と gtk4-layer-shell が必要です。

```bash
# X11/Wayland 双方の対応を有効にするために gtk4 をインストール
sudo emerge --ask gui-libs/gtk gui-libs/gtk4-layer-shell
```

### 2. ビルド
プロジェクトルートで以下を実行し、最適化されたリリースバイナリをビルドします。

```bash
cargo build --release
```

バイナリは `./target/release/glowlay` に生成されます。

---

## 設定ファイル (`config.toml`) の使い方

`glowlay` は TOML 形式の設定ファイルをロードして挙動を変更できます。

### 1. 配置場所と読み込み優先順位
起動時に以下の優先順位で `config.toml` を探索し、最初に見つかったファイルをロードします：

1. **実行バイナリと同じディレクトリ** (例: `/usr/local/bin/config.toml` 等)
2. **現在のカレントディレクトリ** (`./config.toml`)
3. **ユーザー設定ディレクトリ** (`~/.config/glowlay/config.toml` ※推奨)
4. **システム全体の設定ディレクトリ** (`/etc/glowlay/config.toml`)

> [!TIP]
> リリース・常用時には、`~/.config/glowlay/config.toml` に設定ファイルを配置することをおすすめします。

---

### 2. 設定ファイルの内容サンプル
以下はデフォルトで同梱されている設定内容です。

```toml
# glowlay configuration

[window]
width = 210        # ウィジェットの横幅 (ピクセル)
height = 460       # ウィジェットの高さ (ピクセル)
# 表示位置の基準（"top-left", "top-right", "bottom-left", "bottom-right"）
anchor = "top-right"
# 基準点（anchor）からのマージン位置 (x=横マージン, y=縦マージン)
x = 40
y = 40

[style]
# 背景の不透明度（0.0 が完全透明、1.0 が完全不透明。デフォルト 0.4）
bg_alpha = 0.4
# フォントサイズの倍率（1.0 が標準。1.2 で一回り大きくなります）
font_scale = 1.2

[monitor]
# データの更新間隔（ミリ秒）
update_interval_ms = 1000

[features]
# 各モニター機能のON/OFF
enable_cpu = true
enable_mem = true
enable_gpu = true       # NVIDIA GPU非搭載の場合は false にしてください
enable_storage = true
enable_network = true

[network]
# 監視するネットワークインターフェース（空文字 "" の場合は、アクティブなインターフェースを自動検出します）
interface = ""

[storage]
# 監視する複数のマウントポイント（複数指定すると、それぞれの使用量が自動的に並んで表示されます）
paths = ["/", "/home"]
```

---

## 起動方法

バックグラウンドで常駐起動させるには、ターミナルから以下を実行します。

```bash
./target/release/glowlay &
```

自動起動させたい場合は、デスクトップ環境（Sway の `exec` や GNOME/KDE の自動起動アプリケーション）の設定に上記のコマンド（絶対パス指定）を追加してください。

# Rustで学ぶカーネルの低レイヤ

既製のブートローダを使い、カーネルが受け取る情報から学び始めます。画面出力、CPU例外、ページング、アロケータ、割り込み、async、テストの順に進みます。本文のコードは仕組みを示す断片で、単独で起動できる完成カーネルではありません。

前提として、Rustの関数・構造体・参照・列挙型・`Option`・エラー処理に慣れておくと読みやすくなります。ポインタ、`unsafe`、ビット演算、非同期の詳細は必要な章で説明します。

## 読む順番

1. [ブートローダからカーネルへの引き継ぎ](01-bootloader-handoff.md)：入口関数、BootInfo（ブートローダが渡す起動情報）、メモリマップ。
2. [freestanding Rust](02-freestanding-rust.md)：OSのランタイムに依存しないRustプログラム。
3. [フレームバッファコンソール](03-framebuffer-console.md)：ピクセルを通じた文字表示。
4. [CPU例外](04-cpu-exceptions.md)：IDT（CPU例外・割り込みの入口表）とDouble fault。
5. [ページングとメモリ](05-paging-memory.md)：物理メモリと仮想アドレス。
6. [ヒープアロケータ](06-heap-allocator.md)：動的なメモリ確保。
7. [ハードウェア割り込み](07-hardware-interrupts.md)：外部デバイスの通知とFIFO（先入れ先出しのキュー）。
8. [協調的asyncタスク](08-async-tasking.md)：Future（非同期処理の進行状態）と待機処理。
9. [テストとデバッグ](09-testing-debugging.md)：観測と障害の切り分け。

## 資料の読み替え

本文中の各引用 `[n]` は、このMarkdownファイル末尾のSourcesにある同じ番号のURLを指します。土台にはrust-osdev/bootloaderの公式サンプルとAPI資料を使います。[7][8]

`examples/basic`のCargo設定はbootloader 0.11.12とartifact dependencyを示しています。[10][11]

artifact dependencyは、別クレートが生成した実行ファイルをビルド時の入力にするCargo機能です。公式例は実験的な`bindeps`設定とRust nightly（開発版ツールチェーン）を使います。[10][11]

UEFIはファームウェアとOSローダの間で使われる仕様です。公式basic exampleはBIOS用・UEFI用の成果物を扱いますが、この教材の起動確認はUEFI側に限ります。[10][13]

ブートローダ実装、BIOSイメージ作成、ファームウェアからローダーへの移行は課題にしません。既製のbootloaderが担当し、教材はカーネル側から始めます。

カーネル形式とentry pointは公式kernel資料、ディスクイメージ生成はbuild scriptで確認します。[12][13]

bootloaderとAPIの版をそろえ、設定が参照時点から変わっていないか確かめてください。[8][22]

Phil Oppの日本語版は低レイヤ概念の補助資料です。[1]

最小カーネルとテストの記事はBIOSやbootimageを前提にしています。[2][27]

VGAテキストと8259 PICの記事は、それぞれの方式を学ぶ題材として読みます。[3][16]

『ゼロからのOS自作入門』（みかん本）はOS全体の構成を追う参考資料です。[5] C++/EDK II構成のコードや手順は、Rust/bootloader_apiの例へそのまま移しません。

| 資料 | 参考にする内容 | 読み替えの注意 |
|---|---|---|
| Phil Oppシリーズ目次 [1] | 低レイヤ記事の順序 | 各記事の起動方式・依存版を確認 |
| Phil Opp最小カーネル・テスト [2][27] | freestanding・テストの考え方 | BIOS/bootimage手順は使わない |
| Phil Opp VGA・PIC記事 [3][16] | 別方式との比較 | VGA/PIC固有設定を移さない |
| みかん本公式目次 [5] | OS開発全体の章立て | 全31章の代替教材ではない |
| bootloader README/API [7][8] | ローダーとカーネルAPIの契約 | 利用版の対応を確認 |
| basic example設定 [10][11] | ターゲット・artifact dependency | nightly設定は版で変わり得る |
| kernel資料・build script [12][13] | entry point・成果物生成 | 本教材はUEFI経路を選ぶ |
| BootInfo・entry_point [22][28] | 起動情報・関数入口 | API版を合わせる |
| FrameBuffer系API [23][24][26] | 描画バッファの情報 | ピクセル配置を推測しない |
| MemoryRegion系API [29][30] | 領域の境界・列挙 | `end`は排他的 |

みかん本で近い内容を扱う章名は次のとおりです。[5]

| 本教材 | みかん本の関連章 |
|---|---|
| 01 | 第1章「PCの仕組みとハローワールド」、第2章「EDK II入門とメモリマップ」、第3章「画面表示の練習とブートローダ」 |
| 02 | 直接対応なし。Rustのfreestanding環境とABI（コンパイル済みコード間の呼び出し規約など）は本教材独自。 |
| 03 | 第3章「画面表示の練習とブートローダ」、第4章「ピクセル描画とmake入門」、第5章「文字表示とコンソールクラス」 |
| 04 | 第7章「割り込みとFIFO」に関連あり。CPU例外の直接対応ではない。 |
| 05 | 第2章「EDK II入門とメモリマップ」、第8章「メモリ管理」、第19章「ページング」 |
| 06 | 第8章「メモリ管理」 |
| 07 | 第6章「マウス入力とPCI」、第7章「割り込みとFIFO」、第11章「タイマとACPI」、第12章「キー入力」 |
| 08 | 第13章「マルチタスク（1）」、第14章「マルチタスク（2）」 |
| 09 | 目次上で直接対応なし。テスト・デバッグの進め方は本教材独自。 |

## 実行確認と用語

`cargo build`と`cargo run uefi`は公式`examples/basic`ディレクトリ内で実行する手順例です。[10] QEMUはOSイメージを仮想マシンとして起動するエミュレータで、別途用意します。[10]

公式basic-kernelのシリアル出力は、画面描画とは別の診断経路です。`isa-debug-exit`はテスト終了をQEMU側へ伝える仕組みとして記載されています。[12] これらのコマンドや説明をこの環境で実行した結果ではありません。

| 用語 | 説明 |
|---|---|
| UEFI | ファームウェアがOSローダを起動する際に使う仕様。[10][13] |
| BootInfo | ブートローダが渡す起動時情報。メモリ管理器そのものではない。[22] |
| FrameBuffer | 画面の画素領域。形式やstrideを確認して扱う。[23][24][26] |
| 仮想アドレス | ページテーブルを介して物理アドレスへ変換される番地。[17] |
| アロケータ | 空き領域を管理してメモリを貸し出す仕組み。[19][20] |
| Future | `poll`へ`Pending`または`Ready`を返す非同期処理の状態。[21] |
| ABI | 関数の引数や呼び出し方など、バイナリ間の約束。[28] |
| シリアル出力 | 画面とは別の経路からログを送る方法。[12] |

各章では解決したい問題を先に置き、図・式・コードで動作を追います。課題は観測できる結果を確かめながら進めてください。通常のアプリのようなエラー表示を期待できないため、変更前後のログを比べ、どの境界で前提が崩れたかを考えます。

累積学習の概念トレースとして、割り込みFIFOにイベントを記録し、Wakerでexecutorを起こし、タスクがイベントを受け取るまでをつなげて追えます。これは章をまたぐ概念上のトレースであり、実際に動かすには読者自身が用意する起動可能なカーネル土台が必要です。

## Sources

[1] https://os.phil-opp.com/ja — Writing an OS in Rust — Japanese series index
[2] https://os.phil-opp.com/ja/minimal-rust-kernel — Rustでつくる最小のカーネル
[3] https://os.phil-opp.com/ja/vga-text-mode — VGAテキストモード
[5] https://zero.osdev.jp/toc.html — ゼロからのOS自作入門 目次
[7] https://github.com/rust-osdev/bootloader — rust-osdev/bootloader README
[8] https://docs.rs/bootloader_api/0.11.12/bootloader_api — bootloader_api 0.11.12 docs
[10] https://raw.githubusercontent.com/rust-osdev/bootloader/main/examples/basic/basic-os.md
[11] https://raw.githubusercontent.com/rust-osdev/bootloader/main/examples/basic/Cargo.toml
[12] https://raw.githubusercontent.com/rust-osdev/bootloader/main/examples/basic/kernel/basic-kernel.md
[13] https://raw.githubusercontent.com/rust-osdev/bootloader/main/examples/basic/build.rs
[16] https://os.phil-opp.com/ja/hardware-interrupts — ハードウェア割り込み | Writing an OS in Rust
[17] https://os.phil-opp.com/ja/paging-introduction — ページング入門 | Writing an OS in Rust
[19] https://os.phil-opp.com/ja/heap-allocation — ヒープ割り当て | Writing an OS in Rust
[20] https://os.phil-opp.com/allocator-designs/ja — アロケータの設計 | Writing an OS in Rust
[21] https://os.phil-opp.com/ja/async-await — Async/Await | Writing an OS in Rust
[22] https://docs.rs/bootloader_api/0.11.12/bootloader_api/info/struct.BootInfo.html — BootInfo in bootloader_api::info 0.11.12
[23] https://docs.rs/bootloader_api/0.11.12/bootloader_api/info/struct.FrameBuffer.html — FrameBuffer in bootloader_api::info 0.11.12
[24] https://docs.rs/bootloader_api/0.11.12/bootloader_api/info/struct.FrameBufferInfo.html — FrameBufferInfo in bootloader_api::info 0.11.12
[26] https://docs.rs/bootloader_api/0.11.12/bootloader_api/info/enum.PixelFormat.html — PixelFormat in bootloader_api::info 0.11.12
[27] https://os.phil-opp.com/ja/testing — テスト | Writing an OS in Rust
[28] https://docs.rs/bootloader_api/0.11.12/bootloader_api/macro.entry_point.html — entry_point! macro in bootloader_api 0.11.12
[29] https://docs.rs/bootloader_api/0.11.12/bootloader_api/info/struct.MemoryRegion.html — MemoryRegion in bootloader_api::info 0.11.12
[30] https://docs.rs/bootloader_api/0.11.12/bootloader_api/info/struct.MemoryRegions.html — MemoryRegions in bootloader_api::info 0.11.12

# OS開発ロードマップ

このガイドは、OS開発を学びながら、現在のUEFI起動方式を保ってこのRust OSを少しずつ育てるための道筋です。機能を学ぶ順序には『ゼロからのOS自作入門』（以下、みかん本）を使い、Rustでカーネルを書くときの考え方はPhil Oppの「Writing an OS in Rust」日本語版で補います。[1][4][5]

どちらの資料も、掲載されている起動コードやビルド手順をそのままコピーせず、概念を理解します。[2][5][7]

その概念を、このworkspaceの`bootloader`／`bootloader_api`構成へ読み替えます。[8]

## 現在の起動フロー

```text
ビルド時:
kernel (x86_64-unknown-none) → build.rs / bootloader::UefiBoot → uefi.img

実行時:
cargo run → src/main.rs → QEMU + OVMF → uefi.img内のUEFIブートローダー
  → bootloader_api::BootInfoを受け取るkernel → Framebuffer / serialへ出力 → loop
```

- ルートの`Cargo.toml`は`kernel`を含むworkspaceです。ルートのbuild dependencyで`bootloader = 0.11.12`のUEFI featureとkernel artifactを指定し、実行側依存として`ovmf-prebuilt`を使います。
- `build.rs`はkernelのartifactを`bootloader::UefiBoot`へ渡し、`uefi.img`を作ります。
- `src/main.rs`はOVMFを使ってQEMUを起動し、`-display cocoa`と`-serial mon:stdio`を指定します。
- `kernel/Cargo.toml`は`bootloader_api = 0.11.12`などに依存しますが、`uefi` crateには依存していません。カーネル入口は`kernel/src/main.rs`の`no_std`／`no_main`、`entry_point!`、`BootInfo`で構成され、Framebufferを初期化して画面とserialへ起動メッセージを出した後、無限ループします。`bootloader_api`がブートローダーとカーネル間で扱う型を提供します。[8]
- `kernel/src/framebuffer.rs`には、FramebufferへASCII文字を描画する`print!`／`println!`相当のWriterがあります。ここでVGAの`0xb8000`へ書く方式は使いません。

UEFIローダー自体を作ることは、このロードマップの目的ではありません。既存のbootloader構成を保ってカーネルを発展させます。そのため、UEFI crateをカーネルへ追加することは必須ではありません。[7][8]

## 資料の使い分け

Phil Oppのシリーズは、`no_std`のカーネル、出力API、CPU例外、割り込み、paging、heap、allocator、async/awaitなど、Rustカーネルの概念を学ぶ補助資料にします。[1] ただし、同シリーズの最小カーネル記事は本文でUEFI非対応と明記しています。BIOS／`bootimage`／ターゲット設定やVGAテキストバッファのコードは、このリポジトリへそのまま適用できません。[2] VGA記事は文字出力Writerの設計や`print!`／`println!`相当のAPIを考える参考にし、表示先は現在のFramebufferへ置き換えます。[3]

みかん本は、UEFIとブートローダから画面、入力、メモリ、GUI、マルチタスク、端末、FS、アプリへ進む機能の順序を参考にします。[4][5]

MikanOSはEDK II／C++による別実装です。[6]

MikanOSはMikanLoaderPkgも持ちます。[9]

C++コードやEDK IIのビルド手順をこのRust workspaceへそのまま持ち込まず、学ぶ機能と考え方をRustと既存のbootloader構成へ翻訳します。[6]

割り込みを学ぶときは、Phil Oppの例外・割り込み・PICの説明を概念の手がかりにできますが、記事のコードを盲目的にコピーしないでください。UEFIブート後に動くx86_64カーネルという現在の構成に合わせて、初期化状態や割り込みコントローラなどの前提を確かめながら適応します。[1][2]

## 章に沿ったロードマップ

表の「確認」は、その段階で小さな実装を追加した後の目安です。すべての章を一度に作る必要はありません。各段階で動作を確かめ、理解できたところから次へ進みます。みかん本の章番号と章題は公式目次に従っています。[5]

| 章 | 学ぶこと | このリポジトリでの小さな到達目標 | 確認方法・Phil Oppの参考 |
|---|---|---|---|
| 0「OSって個人で作れるの？」 | OSの役割と開発の全体像 | 現在の起動フローと、ファームウェア／ブートローダー／カーネルの責務を説明できる | `cargo build`、`cargo run`で既存の起動を確認。全体像は[みかん本公式目次](https://zero.osdev.jp/toc.html)[5] |
| 1「PCの仕組みとハローワールド」 | PCとUEFI起動、開発環境 | UEFIイメージがQEMU/OVMFから起動し、画面とserialにメッセージを出す流れを追う | 画面とserialの両方でメッセージを見る。Rustのフリースタンディングバイナリは[Phil Opp](https://os.phil-opp.com/ja/)[1]を概念の補助にする |
| 2「EDK II入門とメモリマップ」 | EDK II、メモリマップ、ポインタ | EDK IIのビルドを移植せず、`BootInfo`を通じてカーネルへ渡る情報を調べる | 起動を保ち、画面・serial出力を確認。`BootInfo`の公開情報は[bootloader_api 0.11.12](https://docs.rs/bootloader_api/0.11.12/bootloader_api)[8]を参照 |
| 3「画面表示の練習とブートローダ」 | ピクセル描画、カーネル、ブートローダ | 既存Framebuffer Writerの文字描画を読み、画面表示の小さな変更をする | QEMU画面で描画を確認し、serial出力も確認。ブートローダ構成は[bootloader](https://github.com/rust-osdev/bootloader)[7]を参照 |
| 4「ピクセル描画とmake入門」 | ピクセル操作と描画の整理 | 既存の描画を保ちながら、セル単位で扱うための最小の文字位置管理を検討する | 小さな描画変更をQEMU画面で確認。Framebuffer上の端末設計には[Phil Opp VGA記事](https://os.phil-opp.com/ja/vga-text-mode/)[3]の出力設計を参考にする |
| 5「文字表示とコンソールクラス」 | フォント、文字列、コンソール | Framebufferに文字セル、カーソル、折り返しを導入し、`print!`／`println!` APIを維持する | ASCIIの複数行出力、折り返し、画面端でのスクロールを確認。[Phil Opp VGA記事](https://os.phil-opp.com/ja/vga-text-mode/)[3]は文字出力APIの概念参考にし、VGAメモリ書き込みは使わない |
| 6「マウス入力とPCI」 | PCIデバイスとマウス入力 | まずデバイスや入力の概念を整理し、デバイス列挙または入力の小さな観察目標を決める | serialへ調査結果を記録し、画面表示と起動を保つ。章の順序は[みかん本](https://zero.osdev.jp/toc.html)[5]を参照 |
| 7「割り込みとFIFO」 | 割り込みハンドラ、ベクタ、キュー | 割り込み経路とイベントを分けて考え、受け取ったイベントを記録する段階へ進む | serialでイベントを確認し、割り込み前後も画面表示を保つ。Phil OppのCPU例外・ハードウェア割り込みは概念参考で、UEFI後のx86_64環境に適応する[1] |
| 8「メモリ管理」 | UEFIメモリマップ、スタック、ページング | `BootInfo`から得られるメモリ情報とカーネルのメモリ利用を理解し、小さな管理単位を定める | メモリ情報や動作結果をserialで確認。pagingの考え方は[Phil Opp](https://os.phil-opp.com/ja/)[1]、型の公開情報は[bootloader_api](https://docs.rs/bootloader_api/0.11.12/bootloader_api/)[8]を参照 |
| 9「重ね合わせ処理」 | 画面合成と描画性能 | Framebufferへ描く領域を整理し、画面全体の再描画と部分更新の違いを観察する | 描画前後の画面を比較し、更新箇所を確認。性能測定は後回しにしてよい[5] |
| 10「ウィンドウ」 | ウィンドウ、バックバッファ、移動 | まず固定位置の矩形領域をFramebufferに描く | QEMU画面で背景と矩形領域の重なりを確認[5] |
| 11「タイマとACPI」 | タイマ、時間計測、ACPI | 起動後の時間経過を扱うために必要な情報とタイマの役割を整理する | serialに観測値を出して変化を確認。ACPIやタイマ割り込みは段階を分けて扱う[5] |
| 12「キー入力」 | ACPI情報とキーボード入力 | 入力イベントを受け取る経路を調べ、1種類のキー入力を記録する | serialでキーイベントを確認し、画面出力も保つ[5] |
| 13「マルチタスク（1）」 | コンテキストとタスク切り替え | 2つの処理の状態を区別して説明し、切り替えの最小例を検討する | serial上で処理ごとの進行を区別する。スタックやABIの前提を確認してから実装する[5] |
| 14「マルチタスク（2）」 | スリープ、起床、優先度、アイドル | 待機中の処理とイベントによる再開を小さな目標にする | 待機・再開の順序をserialで確認する[5] |
| 15「ターミナル」 | アクティブウィンドウと端末描画 | 既存のFramebufferコンソールを固定領域へ表示する | QEMU画面で端末領域と出力を確認[5] |
| 16「コマンド」 | 端末入力とコマンド | 入力した文字列を受け取り、最初の簡単なコマンド結果を表示する | 入力から表示までを画面とserialで確認[5] |
| 17「ファイルシステム」 | ボリューム、ディレクトリ、ファイル | ファイルシステムの構造を学び、読み出し対象を限定した小さな目標を決める | 読み出した情報をserialまたは画面で確認する[5] |
| 18「アプリケーション」 | アプリの形式と標準ライブラリ | カーネル機能とアプリの責務を分けて考え、実行方式の要件を整理する | まず実行対象の形式と必要な境界を記録する[5] |
| 19「ページング」 | 仮想アドレス、階層paging、アプリロード | 現在のカーネルのアドレス空間と、アプリに必要な分離を学ぶ | アドレス変換の概念を図にして確認。Phil Oppのpaging記事を補助にする[1] |
| 20「システムコール」 | アプリからOS機能を使う境界 | アプリとカーネルの間で必要な最小の呼び出しを設計する | 呼び出しの入力・結果・境界を記録し、serialで追える形にする[5] |
| 21「アプリからウィンドウ」 | システムコールとアプリ描画 | アプリから文字出力またはウィンドウ操作へ進むための境界を定める | アプリ要求と画面結果の対応を確認する[5] |
| 22「グラフィックとイベント（1）」 | 描画、終了、キーイベント | アプリ相当の描画処理で点や線を表示する | QEMU画面で描画と終了動作を確認[5] |
| 23「グラフィックとイベント（2）」 | マウス、アニメーション、ゲーム | 入力イベントを使った動きのある描画を小さく試す | 画面の変化と入力イベントを確認[5] |
| 24「複数のターミナル」 | 複数端末と複数アプリ | 表示先や処理を複数に分ける設計を整理する | 端末ごとの出力が混ざらないことを画面で確認[5] |
| 25「アプリでファイル読み込み」 | ディレクトリとファイル読み込み | アプリから限定したファイルを読み出す道筋を定める | 内容または読み込み結果を画面・serialで確認[5] |
| 26「アプリでファイル書き込み」 | 標準入力、ファイル記述子、書き込み | 入力からファイルへ渡すデータの流れを小さく設計する | 入出力の結果を確認し、再起動後も必要なら読み出しを確認[5] |
| 27「アプリのメモリ管理」 | demand paging、mmap、copy-on-write | アプリごとのメモリ使用と分離の考え方を整理する | メモリ利用の観測方法を決め、serialで記録する[5] |
| 28「日本語表示とリダイレクト」 | 文字コード、日本語フォント、リダイレクト | ASCII端末からUTF-8文字列・日本語フォントへ進むための課題を分ける | 表示できる文字と代替表示を画面で確認する[5] |
| 29「アプリ間通信」 | 終了コード、パイプ、共有メモリ | 2つの処理間でデータを受け渡す最小の形を考える | 送信側と受信側の結果を確認する[5] |
| 30「おまけアプリ」 | パス、more、cat、ビューア | 既存機能を使う小さなアプリを一つ作る | 実行結果と画面表示を確認する[5] |
| 31「これからの道」 | 次の課題を選ぶ | ここまでで未解決の課題を整理し、次の小目標を一つ決める | できたこと・制約・次の一歩を記録する[5] |

## このリポジトリで次にやるとよいこと

現在はUEFIでの起動と、FramebufferにASCIIを描くWriterまでできています。次はみかん本の4～5章でピクセル描画と文字コンソールの考え方を学び、Phil OppのVGA記事から出力WriterやフォーマットAPIの設計を参考にして、Framebuffer上の文字セル端末へ少しずつ育てるのがよいでしょう。[3][5]

たとえば、まず論理的な列・行とカーソル位置を管理し、次に画面幅で折り返し、セル単位の文字色・背景色を描き、最後に画面下端でスクロールする順に進められます。既存の`print!`／`println!`呼び出しAPIは維持し、描画の中身を段階的に置き換えます。表示先はFramebufferのままにし、VGAの`0xb8000`へ書かない方針です。[3]

## 各段階の進め方

各段階では、次のチェックリストを使います。

- [ ] 該当する章を読み、目的と用語をメモする。[5]
- [ ] BIOS／UEFI固有の起動方法と、CPU・OS一般の概念を分ける。サンプルの起動・ビルド手順をそのまま移さず、現在のbootloader構成に読み替える。[2][6][7]
- [ ] 一度に一つの小さな到達目標だけ実装する。
- [ ] `cargo fmt --all -- --check`、`cargo build`、`cargo run`で確認する。現時点の起動確認は`cargo build`と`cargo run`です。
- [ ] QEMU画面とserialそれぞれの結果を記録し、期待と違う場合は変更を小さく戻して原因を絞る。

カーネルは最後に無限ループするため、`cargo run`やQEMUは自動終了しません。終了するときはQEMUの端末でCtrl+Aの後にxを押します。

## Sources

[1] https://os.phil-opp.com/ja — Writing an OS in Rust — Japanese series index
[2] https://os.phil-opp.com/ja/minimal-rust-kernel — Rustでつくる最小のカーネル
[3] https://os.phil-opp.com/ja/vga-text-mode — VGAテキストモード
[4] https://zero.osdev.jp — ゼロからのOS自作入門 サポートサイト
[5] https://zero.osdev.jp/toc.html — ゼロからのOS自作入門 目次
[6] https://raw.githubusercontent.com/uchan-nos/mikanos/master/README.md — uchan-nos/mikanos README
[7] https://github.com/rust-osdev/bootloader — rust-osdev/bootloader README
[8] https://docs.rs/bootloader_api/0.11.12/bootloader_api — bootloader_api 0.11.12 docs
[9] https://github.com/uchan-nos/mikanos/tree/master/kernel — MikanOS kernel source directory

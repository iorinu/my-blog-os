# 01. ブートローダからカーネルへの引き継ぎ

## カーネルの入口から考える

OSの学習を始めると、電源投入直後にCPUが何をするかをすべて自分で実装したくなります。しかし初期化の段階には、ファームウェア、ブートローダ、カーネルという複数の責任範囲があります。この教材では公式 rust-osdev/bootloader を使い、ブートローダの実装を教材の外に置きます。私たちはカーネルが呼び出された瞬間から始めます。[7][10]

ブートローダはカーネルをメモリに読み込み、実行環境を整え、カーネルが最低限必要とする情報を渡します。この「渡す情報と入口の約束」を引き継ぎ（handoff）と呼びます。ブートローダを使うことは、コンピュータの起動が簡単だという意味ではありません。複雑な段階を既存コンポーネントに委ね、カーネルと周辺機構の境界を学ぶという選択です。

```mermaid
flowchart LR
    F[ファームウェア] --> B[bootloader]
    B -->|カーネルをロードし情報を渡す| K[カーネル入口]
    K --> C[初期化と機能追加]
```

ここでいうファームウェアは、UEFIなど、OSより前に動き機器を初期化するソフトウェアです。ブートローダはファームウェアから制御を受け、カーネルの実行可能形式や設定に応じたロードを担います。UEFIイメージを作る具体的なCargo設定や成果物の組み合わせは公式basic exampleを参照できますが、版によって構成が変わり得ます。[10][11][13]

## BootInfoは約束された境界

`entry_point!`へ渡すカーネル関数は通常のRust ABIを使い、型は `fn(&'static mut BootInfo) -> !` です。`-> !` は通常の値を返さないことを表します。マクロは型を確認し、リンカが呼ぶ `_start` を `extern "C"` の入口として生成します。コールバック関数と生成される入口のABIを区別してください。[22][28]

BootInfoにはメモリ領域の一覧、利用可能ならフレームバッファ、物理メモリの仮想マッピング情報などが含まれます。配置は機種や起動設定ごとに異なり、この構造体はブートローダが調べた環境情報を渡します。構造体は`non-exhaustive`として扱われ、将来フィールドが増える可能性があります。APIが定める型を使い、項目の有無に応じた処理を用意します。[22]

```rust
// 概念例。出力関数は省略しています。
// endがstartより前なら長さを計算しません。
fn inspect_memory(info: &bootloader_api::BootInfo) {
    for region in info.memory_regions.iter() {
        let Some(length) = region.end.checked_sub(region.start) else {
            continue;
        };
        // region.start、region.end、length、region.kindを初期化済み出力先へ記録する。
    }
}
```

`memory_regions`は領域を順に読むためのコレクションです。[22][30] `MemoryRegion`は`start`、排他的な`end`、`kind`を持ちます。長さは`end - start`で求めます。たとえば`start = 0x100000`、`end = 0x300000`なら長さは`0x200000`（2 MiB）で、範囲は`0x100000..0x300000`です。これは読み方の例であり、実際の起動でこの値が現れるという意味ではありません。[29]

このAPIでは、ブートローダが使用した領域をメモリマップに反映し、`Usable`領域はカーネルが利用できると定義されています。[22] `MemoryRegion`の`start`、排他的な`end`、`kind`は領域情報を表します。[29] メモリマップ自体は割り当て状態を管理しません。フレームアロケータを設計するときはページ境界への調整や、再割り当てを避けるため確保済みフレームを追跡する方法を検討します。Phil Oppのページング実装はその設計例として参照できますが、これは例であり、`bootloader_api`が保証するものではありません。[18]

`end`は範囲に含まれない最初のアドレスです。領域を`[start, end)`と表し、長さは`end - start`で求めます。`end`を含む終端だと解釈すると、隣の領域との重複や一バイトのずれが起きます。[29] ページフレームへ分割するときの境界調整は、フレームアロケータ側の設計事項です。

診断ログには`start`、`end`、計算した長さ、`kind`を別々に出します。長さの計算が失敗した領域は空きリストへ入れず、原因を記録します。隣接領域の境界と種別を見比べると、メモリマップを後続のフレームアロケータへ渡す前の不整合を見つけやすくなります。

また、BootInfoを受け取った直後から長時間その参照を保持すべきかは、APIの寿命やカーネルの初期化設計と合わせて考えます。メモリ領域の情報を後でフレームアロケータが必要とするなら、起動後のデータ構造へ安全にコピー・変換する段階が必要です。その変換中に自身が使っているスタックやデータ領域を空き領域と誤認すると、初期化処理そのものが壊れます。ブートローダが使用領域をマップに反映するというAPI上の情報と、カーネルが以後利用する領域の管理方針を分けて記録しましょう。

## まず境界を観察する

起動直後に大きな機能を追加するより、引数が届いたこと、メモリ領域が列挙できること、画面情報が存在するかを記録するのが安全です。公式basic kernelもBootInfoの受け取りや情報の出力を最初の観察点にしています。[12] 出力先はシリアルやフレームバッファなど選べますが、どの手段も初期化状態や実行環境に依存します。後の章で画面と診断手段を段階的に作ります。

起動引き継ぎは、ブートローダが「OSの初期化を完了した」と約束するものではありません。カーネルが使うスタックやページテーブルの状態を、対応するAPIと設定で調べます。割り込み状態や装置の初期状態も、確認すべき引き継ぎ項目です。ある環境で偶然動いた前提を標準仕様と思い込むと、別のファームウェアやブート設定で再現しない問題になります。BootInfoに記載された情報の範囲で判断します。

よくある誤解は、BootInfoがカーネルのメモリ管理器そのものだと思うことです。BootInfoは起動時の事実を伝える情報であり、どのページを割り当てるか、解放をどう追跡するかを決めるのはカーネルです。また、`physical_memory_offset` があれば全物理アドレスを自由にポインタ化してよいわけでもありません。マッピングと所有権を理解せずに別名参照すると、Rustの安全性を壊し得ます。[22]

## 手を動かす

BootInfoのAPI資料を開き、各領域の`start`、排他的な`end`、`kind`をログに出します。長さは`end.checked_sub(start)`で求め、範囲を図にしてください。隣り合う領域と`Usable`以外の領域を区別し、フレームバッファがない場合も処理を続けられるようにします。[22][29] メモリマップは実行環境ごとに変わるため、特定のアドレスになることを成功条件にしません。

Phil Oppの最小カーネル記事は起動直後のカーネルを考える補助になります。記事にあるBIOS/bootimageの流れは、ここでの実行手順には使いません。[2] 次章では、入口関数が動くRustプログラムからOSの標準機能を取り除くと何が変わるのかを見ます。

## Sources

[2] https://os.phil-opp.com/ja/minimal-rust-kernel — Rustでつくる最小のカーネル
[7] https://github.com/rust-osdev/bootloader — rust-osdev/bootloader README
[10] https://raw.githubusercontent.com/rust-osdev/bootloader/main/examples/basic/basic-os.md
[11] https://raw.githubusercontent.com/rust-osdev/bootloader/main/examples/basic/Cargo.toml
[12] https://raw.githubusercontent.com/rust-osdev/bootloader/main/examples/basic/kernel/basic-kernel.md
[13] https://raw.githubusercontent.com/rust-osdev/bootloader/main/examples/basic/build.rs
[18] https://os.phil-opp.com/ja/paging-implementation — ページングの実装 | Writing an OS in Rust
[22] https://docs.rs/bootloader_api/0.11.12/bootloader_api/info/struct.BootInfo.html — BootInfo in bootloader_api::info 0.11.12
[28] https://docs.rs/bootloader_api/0.11.12/bootloader_api/macro.entry_point.html — entry_point! macro in bootloader_api 0.11.12
[29] https://docs.rs/bootloader_api/0.11.12/bootloader_api/info/struct.MemoryRegion.html — MemoryRegion in bootloader_api::info 0.11.12
[30] https://docs.rs/bootloader_api/0.11.12/bootloader_api/info/struct.MemoryRegions.html — MemoryRegions in bootloader_api::info 0.11.12

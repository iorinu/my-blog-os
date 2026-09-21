# my-blog-os

[Writing an OS in Rust](https://os.phil-opp.com/ja/)を参考に、自作OSを学習しています。

現在は`bootloader 0.11`のUEFIブート構成を使っています。

## 実行

```bash
cargo build
cargo run
```

`cargo run`はOVMFを使ってUEFIモードのQEMUを起動し、シリアル出力にカーネルの起動メッセージを表示します。

UEFIディスクイメージは、`cargo build`の出力に表示される`target/debug/build/*/out/uefi.img`です。USBへ書き込む場合は、ファイルをコピーするのではなく、ディスク全体へraw書き込みします。

現在のVGAテキストバッファ出力は、UEFIのframebuffer出力へ移行する前段階としてkernelクレートに残しています。

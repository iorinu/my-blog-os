# UEFIイメージをUSBへ更新する手順

macOSで自作OSのUEFIイメージをUSBへ更新する手順です。

この環境ではUSBのrawデバイスへの`dd`が`Operation not permitted`になるため、UEFIイメージのEFIパーティションとUSBをそれぞれマウントし、Finderで必要なファイルをコピーします。

## 1. UEFIイメージを生成する

リポジトリのルートで実行します。

```bash
cargo build
```

出力に表示された次の形式のパスを確認します。

```text
target/debug/build/<package-hash>/out/uefi.img
```

以降では、次のパスを例として使います。`<package-hash>`は実際の出力に置き換えてください。

```text
/Users/iori/src/github.com/iorinu/my-blog-os/target/debug/build/<package-hash>/out/uefi.img
```

## 2. USBのデバイス番号を確認する

USBを接続した状態で実行します。

```bash
diskutil list
```

容量や`external, physical`という表示を確認して、USB本体のデバイス番号を特定します。

```text
/dev/disk4 (external, physical):
```

以下ではUSB本体を`/dev/disk4`、パーティションを`/dev/disk4s1`として説明します。実際の表示が異なる場合は、以降のコマンドで置き換えてください。

デバイス番号はUSBを接続し直すと変わることがあります。以前の番号をそのまま使用しないでください。

## 3. USBとUEFIイメージをマウントする

以前マウントしたUEFIイメージがあれば取り外します。

```bash
hdiutil detach /dev/disk5 2>/dev/null || true
```

USBのEFIパーティションをマウントします。

```bash
diskutil mount /dev/disk4s1
```

UEFIイメージをパーティション単位で接続します。

```bash
hdiutil attach -readonly -nomount \
  "/Users/iori/src/github.com/iorinu/my-blog-os/target/debug/build/<package-hash>/out/uefi.img"
```

出力例：

```text
/dev/disk5
/dev/disk5s1
```

出力されたEFIパーティションを読み取り専用でマウントします。

```bash
diskutil mount readOnly /dev/disk5s1
```

通常は次のマウントポイントになります。

```text
/Volumes/ESP       # USB側
/Volumes/kernel-9751  # UEFIイメージ側
```

マウントポイントが異なる場合は、Finderで表示された実際の名前を使ってください。

## 4. Finderでファイルを上書きする

2つのボリュームをFinderで開きます。

```bash
open /Volumes/ESP
open /Volumes/kernel-9751
```

UEFIイメージ側からUSB側へ、次の2ファイルをコピーします。

```text
コピー元:
/Volumes/kernel-9751/efi/boot/bootx64.efi
/Volumes/kernel-9751/kernel-x86_64

コピー先:
/Volumes/ESP/EFI/BOOT/BOOTX64.EFI
/Volumes/ESP/kernel-x86_64
```

`/Volumes/ESP/EFI/BOOT`が存在しない場合は、Finderで`EFI`と`BOOT`のフォルダを作成します。

既存ファイルの置き換え確認が表示されたら、「置き換える」を選択します。

Terminalや`sudo cp`ではUSBへの書き込みが拒否されることがあるため、現在の環境ではFinderでコピーします。

## 5. 取り外す

コピーが完了したことをFinderで確認してから、UEFIイメージを取り外します。

```bash
hdiutil detach /dev/disk5
```

最後にUSBを取り出します。

```bash
diskutil eject /dev/disk4
```

`hdiutil attach`で`disk5`以外が割り当てられた場合は、実際に表示されたイメージ側のデバイス番号を使ってください。

## 6. 実機で起動する

1. USBを実機へ接続する
2. UEFIブートメニューを開く
3. USBのUEFIエントリを選択する
4. `Hello World!`が画面左上に表示されることを確認する

Framebuffer対応後のカーネルは、UEFIから渡されたFramebufferへ`Hello World!`を描画します。画面にUEFIブートローダーのログが残ったままで`Hello World!`が表示されない場合は、修正版の2ファイルがUSBへ上書きされているか確認してください。

## 注意事項

- `diskutil list`でUSBのデバイス番号を毎回確認する。
- `/dev/disk4`や`/dev/disk5`を別のディスクに置き換えない。
- この手順ではUSB全体を消去しない。
- `dd`によるraw書き込みは、この環境では`Operation not permitted`になるため使用しない。
- Secure Bootが有効な実機では、署名されていないUEFIブートローダーが拒否される場合がある。
- USBを取り外す前に、必ず`diskutil eject`を実行する。

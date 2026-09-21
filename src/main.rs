use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::env;
use std::process::Command;

fn main() {
    let uefi_path = env!("UEFI_PATH");
    let prebuilt = Prebuilt::fetch(Source::LATEST, "target/ovmf")
        .expect("OVMFファームウェアを取得できませんでした");
    let code = prebuilt.get_file(Arch::X64, FileType::Code);
    let vars = prebuilt.get_file(Arch::X64, FileType::Vars);

    let mut command = Command::new("qemu-system-x86_64");
    command.arg("-serial").arg("mon:stdio");
    command.arg("-display").arg("none");
    command
        .arg("-drive")
        .arg(format!("format=raw,file={uefi_path}"));
    command.arg("-drive").arg(format!(
        "if=pflash,format=raw,unit=0,file={},readonly=on",
        code.display()
    ));
    command.arg("-drive").arg(format!(
        "if=pflash,format=raw,unit=1,file={},snapshot=on",
        vars.display()
    ));

    command
        .status()
        .expect("qemu-system-x86_64を起動できませんでした");
}

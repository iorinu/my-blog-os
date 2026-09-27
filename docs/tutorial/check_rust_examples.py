#!/usr/bin/env python3
"""チュートリアルの指定Rustコードを個別にテスト実行する。"""

from pathlib import Path
import re
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parent
FENCE = re.compile(r"^```rust\s*$\n(.*?)^```\s*$", re.MULTILINE | re.DOTALL)


def main() -> int:
    blocks = []
    for path in sorted(ROOT.glob("*.md")):
        text = path.read_text(encoding="utf-8")
        for number, match in enumerate(FENCE.finditer(text), start=1):
            code = match.group(1)
            if code.splitlines() and code.splitlines()[0].strip() == "// tutorial:compile":
                blocks.append((path, number, code.split("\n", 1)[1]))

    if not blocks:
        print("tutorial:compile マーカー付きRustブロックがありません", file=sys.stderr)
        return 1

    failed = False
    with tempfile.TemporaryDirectory(prefix="tutorial-rust-") as temp:
        temp_dir = Path(temp)
        for index, (path, number, code) in enumerate(blocks):
            source = temp_dir / f"example_{index}.rs"
            binary = temp_dir / f"example_{index}"
            source.write_text(code, encoding="utf-8")
            compile_result = subprocess.run(
                ["rustc", "--edition=2021", "--test", str(source), "-o", str(binary)],
                text=True, capture_output=True,
            )
            if compile_result.returncode:
                failed = True
                print(f"失敗: {path.name} Rustブロック{number} (コンパイル)\n{compile_result.stderr}", file=sys.stderr)
                continue
            test_result = subprocess.run([str(binary)], text=True, capture_output=True)
            if test_result.returncode:
                failed = True
                print(f"失敗: {path.name} Rustブロック{number} (テスト)\n{test_result.stdout}{test_result.stderr}", file=sys.stderr)

    if failed:
        return 1
    print(f"成功: {len(blocks)}個のRustブロックをコンパイルし、テストしました")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

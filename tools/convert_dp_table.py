#!/usr/bin/env python3
"""
DPテーブル変換ツール

yacht_dp_table.hpp からdouble値を抽出し、
f32バイナリ (little-endian) として出力する。

構造: dp_table[upper_sum][used_hands]
- upper_sum: 0-63 (64通り)
- used_hands: 0-4095 (4096通り)
- 合計: 64 * 4096 = 262,144 エントリ

出力: dp_table.bin (262,144 * 4 = 1,048,576 bytes = 1MB)
"""

import re
import struct
import sys
from pathlib import Path


def parse_hpp_file(filepath: Path) -> list[list[float]]:
    """C++ヘッダーファイルからDP値を抽出"""
    content = filepath.read_text()

    # 配列部分を抽出（{{ から }} まで）
    pattern = r'dp_table\s*=\s*\{([\s\S]*?)\};'
    match = re.search(pattern, content)
    if not match:
        raise ValueError("dp_table が見つかりませんでした")

    array_content = match.group(1)

    # 各行（upper_sum ごと）を処理
    dp_table = []
    row_pattern = r'\{([^}]+)\}'

    for row_match in re.finditer(row_pattern, array_content):
        row_str = row_match.group(1)
        values = [float(v.strip()) for v in row_str.split(',') if v.strip()]
        dp_table.append(values)

    return dp_table


def validate_table(dp_table: list[list[float]]) -> None:
    """テーブルの構造を検証"""
    expected_rows = 64
    expected_cols = 4096

    if len(dp_table) != expected_rows:
        raise ValueError(f"行数が不正: 期待={expected_rows}, 実際={len(dp_table)}")

    for i, row in enumerate(dp_table):
        if len(row) != expected_cols:
            raise ValueError(f"行{i}の列数が不正: 期待={expected_cols}, 実際={len(row)}")

    # 初期期待値を確認
    initial_score = dp_table[0][0]
    print(f"初期期待値: {initial_score:.6f}")

    if not (180 < initial_score < 200):
        print(f"警告: 初期期待値が予想範囲外です")


def write_binary(dp_table: list[list[float]], output_path: Path) -> None:
    """f32 little-endian バイナリとして出力"""
    with output_path.open('wb') as f:
        for row in dp_table:
            for value in row:
                # double -> float32 に変換して書き込み
                f.write(struct.pack('<f', value))


def main():
    # パス設定
    script_dir = Path(__file__).parent
    input_path = script_dir / 'yacht_dp_table.hpp'
    output_path = script_dir.parent / 'yacht-core' / 'src' / 'dp_table.bin'

    if not input_path.exists():
        print(f"エラー: {input_path} が見つかりません")
        sys.exit(1)

    print(f"入力: {input_path}")
    print(f"出力: {output_path}")

    # 変換処理
    print("DP テーブルを解析中...")
    dp_table = parse_hpp_file(input_path)

    print("テーブル構造を検証中...")
    validate_table(dp_table)

    print("バイナリ出力中...")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    write_binary(dp_table, output_path)

    # 結果表示
    size_bytes = output_path.stat().st_size
    print(f"完了: {size_bytes:,} bytes ({size_bytes / 1024 / 1024:.2f} MB)")

    # サンプル値を表示
    print("\nサンプル値:")
    print(f"  dp[0][0] = {dp_table[0][0]:.6f}")
    print(f"  dp[0][4095] = {dp_table[0][4095]:.6f}")
    print(f"  dp[63][0] = {dp_table[63][0]:.6f}")
    print(f"  dp[63][4095] = {dp_table[63][4095]:.6f}")


if __name__ == '__main__':
    main()

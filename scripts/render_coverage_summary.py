#!/usr/bin/env python3
from pathlib import Path

rows = []
for line in Path("target/llvm-cov/summary.txt").read_text().splitlines():
    line = line.strip()
    if not line or not (line.startswith("/") or line.startswith("TOTAL")):
        continue

    parts = line.split()
    rows.append(
        {
            "file": parts[0],
            "regions": parts[3],
            "functions": parts[6],
            "lines": parts[9],
        }
    )

print("## Coverage Summary")
print()
print("| File | Regions | Functions | Lines |")
print("| --- | ---: | ---: | ---: |")
for row in rows:
    print(
        f"| `{row['file']}` | {row['regions']} | {row['functions']} | {row['lines']} |"
    )

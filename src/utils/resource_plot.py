#!python

import re
import sys
import matplotlib.pyplot as plt
from datetime import datetime
from pathlib import Path

def extract_used_memory(path):
    timestamps = []
    used_mem = []

    with open(path, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()

    for match in re.finditer(
        r'(\d{4}-\d{2}-\d{2}T[\d:.]+Z).*?TRACE rx:\s+(\d+)\s+(\d+)\s+(\d+)',
        content
    ):
        ts_str, _, _, used = match.groups()

        timestamps.append(datetime.fromisoformat(ts_str.replace("Z", "+00:00")))
        used_mem.append(int(used))

    if not timestamps:
        return None, None

    # Normalize time to start at 0
    t0 = min(timestamps)
    elapsed = [(t - t0).total_seconds() for t in timestamps]

    return elapsed, used_mem

def main(output_path: str, inputs: list[str]):
    plt.figure()

    for input in inputs:
        data = input.split(":")

        if len(data) == 1:
            path = data[0]
            label = Path(path).stem
        else:
            label = data[0]
            path = data[1]

        elapsed, used = extract_used_memory(path)

        if elapsed is None:
            print(f"Warning: no data found in {path}")
            continue

        plt.plot(elapsed, used, label=label)

    plt.xlabel("Time (seconds since experiment start)")
    plt.ylabel("Used Memory (MiB)")
    plt.title("Used Memory Comparison")
    plt.legend()
    plt.tight_layout()
    plt.savefig(Path(output_path).with_suffix('.svg'))
    plt.savefig(Path(output_path).with_suffix('.png'))


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <output_path> <log1> <name:log2> <test:log3> ...")
        sys.exit(1)

    main(sys.argv[1], sys.argv[2:])
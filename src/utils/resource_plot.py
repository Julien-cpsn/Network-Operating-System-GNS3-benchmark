#!python

import re
import sys
import matplotlib.pyplot as plt
from matplotlib.legend_handler import HandlerTuple
from datetime import datetime
from pathlib import Path

def extract_used_resources(path):
    timestamps = []
    used_mem = []
    cpu_usage = []

    with open(path, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()

    for match in re.finditer(
        r'(\d{4}-\d{2}-\d{2}T[\d:.]+Z).*?TRACE rx:\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)',
        content
    ):
        ts_str, _, _, used, cpu = match.groups()

        cpu = float(cpu)
        if cpu < 0 or cpu > 100:
            cpu = 0

        timestamps.append(datetime.fromisoformat(ts_str.replace("Z", "+00:00")))
        used_mem.append(float(used))
        cpu_usage.append(cpu)

    if not timestamps:
        return None, None

    # Normalize time to start at 0
    t0 = min(timestamps)
    elapsed = [(t - t0).total_seconds() for t in timestamps]

    return elapsed, used_mem, cpu_usage

def main(output_path: str, inputs: list[str]):
    fig, ax1 = plt.subplots(figsize=(7, 4))
    ax2 = ax1.twinx()

    mem_lines = []
    cpu_lines = []

    for input in inputs:
        data = input.split(":")

        if len(data) == 1:
            path = data[0]
            label = Path(path).stem
        else:
            label = data[0]
            path = data[1]

        elapsed, used_memory, cpu_usage = extract_used_resources(path)

        if elapsed is None:
            print(f"Warning: no data found in {path}")
            continue

        mem_line, = ax1.plot(elapsed, used_memory, label=label)
        cpu_line, = ax2.plot(elapsed, cpu_usage, linestyle=":", color=mem_line.get_color())

        mem_lines.append(mem_line)
        cpu_lines.append(cpu_line)

    ax1.set_xlabel("Time (seconds since experiment start)")
    ax1.set_ylabel("Used Memory (MiB)")
    ax2.set_ylabel("CPU load (%)")

    ax1.set_ylim(bottom=0)
    ax2.set_ylim(0, 100)

    plt.title("Used Memory and CPU load Comparison")

    # Combine legends
    legend_handles = [
        (m, c) for m, c in zip(mem_lines, cpu_lines)
    ]
    legend_labels = [m.get_label() for m in mem_lines]

    ax1.legend(
        legend_handles,
        legend_labels,
        handler_map={tuple: HandlerTuple(ndivide=None)},
        loc="center left",
        bbox_to_anchor=(1.15, 0.85),
        borderaxespad=0,
        frameon=False
    )

    plt.tight_layout(rect=(0, 0, 1.05, 1))
    plt.savefig(Path(output_path).with_suffix('.svg'))
    plt.savefig(Path(output_path).with_suffix('.png'))


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <output_path> <log1> <name:log2> <test:log3> ...")
        sys.exit(1)

    main(sys.argv[1], sys.argv[2:])
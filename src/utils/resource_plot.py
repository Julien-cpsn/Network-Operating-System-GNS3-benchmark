#!python

import re
import sys
import matplotlib.pyplot as plt
from matplotlib.legend_handler import HandlerTuple
from datetime import datetime, timedelta
from pathlib import Path

def extract_experiment_time(path: str) -> tuple[datetime, datetime]:
    parent_folder = Path(path).parent
    experiment_log_file_path = f'{parent_folder}/experiment.log'

    with open(experiment_log_file_path, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()

        start_match = re.findall(r'(\d{4}-\d{2}-\d{2}T[\d:.]+Z)\s*INFO experiment: Experiment start', content)
        start_time = datetime.fromisoformat(start_match[0].replace("Z", "+00:00"))

        end_match = re.findall(r'(\d{4}-\d{2}-\d{2}T[\d:.]+Z)\s*INFO experiment: Experiment end', content)
        end_time = datetime.fromisoformat(end_match[0].replace("Z", "+00:00"))

        return start_time, end_time

def extract_used_resources(path: str, start_time: datetime, end_time: datetime):
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

    measurement_start = start_time - timedelta(seconds=5)
    measurement_end = end_time + timedelta(seconds=5)

    filtered = [
        (t, mem, cpu)
        for t, mem, cpu in zip(timestamps, used_mem, cpu_usage)
        if measurement_start <= t <= measurement_end
    ]

    if not filtered:
        return None, None, None

    timestamps, used_mem, cpu_usage = map(list, zip(*filtered))

    # Normalize time to start at 0
    elapsed = [(t - start_time).total_seconds() for t in timestamps]

    return elapsed, used_mem, cpu_usage

def main(output_path: str, inputs: list[str]):
    fig, ax1 = plt.subplots(figsize=(7, 4))
    ax2 = ax1.twinx()

    mem_lines = []
    cpu_lines = []

    experiment_start = None
    experiment_end = None

    for input in inputs:
        data = input.split(":")

        if len(data) == 1:
            path = data[0]
            label = Path(path).stem
        else:
            label = data[0]
            path = data[1]

        start_time, end_time = extract_experiment_time(path)
        elapsed, used_memory, cpu_usage = extract_used_resources(path, start_time, end_time)

        if experiment_start is None:
            experiment_start = start_time
            experiment_end = end_time

        if elapsed is None:
            print(f"Warning: no data found in {path}")
            continue

        mem_line, = ax1.plot(elapsed, used_memory, label=label)
        cpu_line, = ax2.plot(elapsed, cpu_usage, linestyle=":", color=mem_line.get_color())

        mem_lines.append(mem_line)
        cpu_lines.append(cpu_line)

    # Convert experiment end time to seconds relative to experiment start
    experiment_end_elapsed = (experiment_end - experiment_start ).total_seconds()

    # Experiment boundaries
    ax1.axvline(x=0, color="grey", linestyle="--", linewidth=1)
    ax1.axvline(x=experiment_end_elapsed, color="grey", linestyle="--", linewidth=1)

    ax1.text(0, 0.6, "t0", transform=ax1.get_xaxis_transform(), rotation=90, va="top", ha="right", color="grey")
    ax1.text(experiment_end_elapsed + 1, 0.6, "t_end", transform=ax1.get_xaxis_transform(), rotation=90, va="top", ha="left", color="grey")

    ax1.set_xlabel("Time (seconds since experiment start)")
    ax1.set_ylabel("— Used Memory (MiB)")
    ax2.set_ylabel("···  CPU load (%)")

    ax1.set_ylim(bottom=0)
    ax2.set_ylim(0, 100)

    plt.title("Used Memory and CPU load")

    # Combine legends
    legend_handles = [
        (m, c) for m, c in zip(mem_lines, cpu_lines)
    ]
    legend_labels = [m.get_label() for m in mem_lines]

    max_label = len(max(legend_labels, key=len))

    ax1.legend(
        legend_handles,
        legend_labels,
        handler_map={tuple: HandlerTuple(ndivide=None)},
        loc="center left",
        bbox_to_anchor=(1.025 + max_label * 0.005, 0.85),
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
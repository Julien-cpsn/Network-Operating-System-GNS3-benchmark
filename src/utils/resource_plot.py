#!python

import re
import sys
import matplotlib.pyplot as plt
from matplotlib.legend_handler import HandlerTuple
from datetime import datetime, timedelta
from pathlib import Path

from matplotlib.patches import Ellipse


def extract_experiment_time(path: str) -> tuple[datetime, datetime]:
    parent_folder = Path(path).parent
    experiment_log_file_path = f'{parent_folder}/experiment.log'

    with open(experiment_log_file_path, "r", encoding="utf-8", errors="ignore") as f:
        content = f.read()

        start_match = re.findall(r'(\d{4}-\d{2}-\d{2}T[\d:.]+Z)\s*INFO experiment: Experiment start', content)
        start_time = datetime.fromisoformat(start_match[len(start_match) - 1].replace("Z", "+00:00"))

        end_match = re.findall(r'(\d{4}-\d{2}-\d{2}T[\d:.]+Z)\s*INFO experiment: Experiment end', content)
        end_time = datetime.fromisoformat(end_match[len(end_match) - 1].replace("Z", "+00:00"))

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

    measurement_start = start_time - timedelta(seconds=10)
    measurement_end = end_time + timedelta(seconds=10)

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
    fig, ax1 = plt.subplots(figsize=(7, 4), dpi=150)
    ax2 = ax1.twinx()

    ellipse_fig, ellipse_ax = plt.subplots(figsize=(6, 5), dpi=150)
    all_cpu_values = []


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

        # --------------------------------------------------------------
        # CPU / memory ellipse
        # --------------------------------------------------------------
        cpu_min = min(cpu_usage)
        cpu_max = max(cpu_usage)
        mem_min = min(used_memory)
        mem_max = max(used_memory)

        cpu_center = (cpu_min + cpu_max) / 2
        mem_center = (mem_min + mem_max) / 2

        cpu_range = cpu_max - cpu_min
        mem_range = mem_max - mem_min

        all_cpu_values.extend(cpu_usage)

        ellipse = Ellipse(
            xy=(cpu_center, mem_center),
            width=cpu_range,
            height=mem_range,
            angle=0,
            facecolor=mem_line.get_color(),
            edgecolor=mem_line.get_color(),
            alpha=0.20,
            linewidth=1.5,
            label=label
        )

        ellipse_ax.add_patch(ellipse)

        # Mark the center of the min/max range
        ellipse_ax.plot(cpu_center, mem_center, marker=".", color=mem_line.get_color())

    # ------------------------------------------------------------------
    # Experiment boundaries
    # ------------------------------------------------------------------

    # Convert experiment end time to seconds relative to experiment start
    experiment_end_elapsed = (experiment_end - experiment_start ).total_seconds()

    # Experiment boundaries
    ax1.axvline(x=0, color="grey", linestyle="--", linewidth=1)
    ax1.axvline(x=experiment_end_elapsed, color="grey", linestyle="--", linewidth=1)

    ax1.text(0, 0.6, "t0", transform=ax1.get_xaxis_transform(), rotation=90, va="top", ha="right", color="grey")
    ax1.text(experiment_end_elapsed + 1, 0.6, "t_end", transform=ax1.get_xaxis_transform(), rotation=90, va="top", ha="left", color="grey")

    # ------------------------------------------------------------------
    # Time-series plot formatting
    # ------------------------------------------------------------------

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
        bbox_to_anchor=(1.17, 0.85 - len(legend_labels) * 0.02, 0.0, 0.0),
        borderaxespad=0,
        frameon=False
    )

    fig.tight_layout(rect=(0, 0, 1.0 + 0.0035 * max_label, 1))

    # ------------------------------------------------------------------
    # Ellipse plot formatting
    # ------------------------------------------------------------------

    ellipse_ax.set_xlabel("CPU load (%)")
    ellipse_ax.set_ylabel("Used Memory (MiB)")
    ellipse_ax.set_title("CPU load vs. Used Memory")

    # Adaptive CPU axis based on the actual data.
    cpu_min_all = min(all_cpu_values)
    cpu_max_all = max(all_cpu_values)

    cpu_range = cpu_max_all - cpu_min_all
    cpu_margin = max(cpu_range * 0.05, 1.0)

    ellipse_ax.set_xlim(
        max(0, cpu_min_all - cpu_margin),
        cpu_max_all + cpu_margin
    )
    ellipse_ax.set_ylim(bottom=0)

    # Use the experiment names for the ellipse legend
    ellipse_ax.legend(loc="center left", bbox_to_anchor=(1.0, 0.85), frameon=False )
    ellipse_ax.grid(True, alpha=0.2)

    ellipse_fig.tight_layout()

    # ------------------------------------------------------------------
    # Save both plots
    # ------------------------------------------------------------------
    output_path = Path(output_path)

    fig.savefig(Path(output_path).with_suffix('.svg'))
    fig.savefig(Path(output_path).with_suffix('.png'))
    ellipse_fig.savefig(output_path.with_name(output_path.stem + "_ellipse.svg"))
    ellipse_fig.savefig(output_path.with_name(output_path.stem + "_ellipse.png"))


if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <output_path> <log1> <name:log2> <test:log3> ...")
        sys.exit(1)

    main(sys.argv[1], sys.argv[2:])
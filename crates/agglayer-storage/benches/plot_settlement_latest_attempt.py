#!/usr/bin/env python3
"""Generate benchmark graphs for settlement latest-attempt strategies.

Input CSV columns:
dataset,workload,strategy,ops_per_s,write_p50_us,write_p95_us,read_p50_us,read_p95_us,db_size_mb,db_size_delta_pct
"""

from __future__ import annotations

import argparse
import csv
from pathlib import Path

try:
    import matplotlib.pyplot as plt
except ModuleNotFoundError as exc:
    raise SystemExit(
        "matplotlib is required to generate benchmark graphs. "
        "Install it with: python3 -m pip install matplotlib"
    ) from exc


STRATEGY_COLORS = {
    "strategy_prefix_scan": "#1f77b4",
    "strategy_latest_cf": "#ff7f0e",
}

DATASET_COLORS = {
    "ds_1k_realistic_1_25": "#4c78a8",
    "ds_10k_realistic_1_25": "#f58518",
    "ds_50k_realistic_1_25": "#54a24b",
}

STRATEGY_LABELS = {
    "strategy_prefix_scan": "Prefix Scan",
    "strategy_latest_cf": "Latest CF",
}

DATASET_ORDER = [
    "ds_1k_realistic_1_25",
    "ds_10k_realistic_1_25",
    "ds_50k_realistic_1_25",
]

DATASET_LABELS = {
    "ds_1k_realistic_1_25": "1k jobs",
    "ds_10k_realistic_1_25": "10k jobs",
    "ds_50k_realistic_1_25": "50k jobs",
}

WORKLOAD_ORDER = [
    "wl_write_only",
    "wl_write_heavy_90_10",
    "wl_read_only_latest_hit",
    "wl_read_only_latest_miss",
]

WORKLOAD_LABELS = {
    "wl_write_only": "Write Only",
    "wl_write_heavy_90_10": "Write 90/Read 10",
    "wl_read_only_latest_hit": "Read Hit",
    "wl_read_only_latest_miss": "Read Miss",
}

WORKLOAD_SHORT = {
    "wl_write_only": "WO",
    "wl_write_heavy_90_10": "W90R10",
    "wl_read_only_latest_hit": "RH",
    "wl_read_only_latest_miss": "RM",
}


def short_combo_label(dataset: str, workload: str) -> str:
    dataset_short = {
        "ds_1k_realistic_1_25": "1k",
        "ds_10k_realistic_1_25": "10k",
        "ds_50k_realistic_1_25": "50k",
    }.get(dataset, dataset)
    workload_short = WORKLOAD_SHORT.get(workload, workload)
    return f"{dataset_short}-{workload_short}"


def workload_legend_text() -> str:
    return "WO=Write Only, W90R10=Write 90% / Read 10%, RH=Read Hit, RM=Read Miss"


def parse_float(value: str) -> float | None:
    value = value.strip()
    if not value or value == "baseline":
        return None
    return float(value)


def load_rows(csv_path: Path) -> list[dict[str, str]]:
    with csv_path.open("r", newline="", encoding="utf-8") as f:
        return list(csv.DictReader(f))


def grouped_keys(rows: list[dict[str, str]]) -> list[tuple[str, str]]:
    keys = {(r["dataset"], r["workload"]) for r in rows}
    sorted_keys = []
    for dataset in DATASET_ORDER:
        for workload in WORKLOAD_ORDER:
            key = (dataset, workload)
            if key in keys:
                sorted_keys.append(key)
    return sorted_keys


def value_map(rows: list[dict[str, str]], metric: str) -> dict[tuple[str, str, str], float | None]:
    result: dict[tuple[str, str, str], float | None] = {}
    for row in rows:
        result[(row["dataset"], row["workload"], row["strategy"])] = parse_float(row[metric])
    return result


def plot_grouped_bars(
    rows: list[dict[str, str]],
    metric: str,
    title: str,
    ylabel: str,
    output_path: Path,
) -> None:
    values = value_map(rows, metric)
    width = 0.35

    fig, axes = plt.subplots(1, len(DATASET_ORDER), figsize=(18, 6), sharey=True)
    if len(DATASET_ORDER) == 1:
        axes = [axes]

    for axis, dataset in zip(axes, DATASET_ORDER):
        workloads = [
            workload
            for workload in WORKLOAD_ORDER
            if values.get((dataset, workload, "strategy_prefix_scan")) is not None
            or values.get((dataset, workload, "strategy_latest_cf")) is not None
        ]

        x = list(range(len(workloads)))
        has_data = False

        for idx, strategy in enumerate(["strategy_prefix_scan", "strategy_latest_cf"]):
            xs = [p + (idx - 0.5) * width for p in x]
            ys = [values.get((dataset, workload, strategy)) for workload in workloads]
            ys_clean = [float("nan") if y is None else y for y in ys]
            has_data = has_data or any(y is not None for y in ys)
            axis.bar(
                xs,
                ys_clean,
                width,
                label=STRATEGY_LABELS[strategy],
                color=STRATEGY_COLORS[strategy],
            )

        if not has_data:
            axis.text(0.5, 0.5, "No data", transform=axis.transAxes, ha="center", va="center")

        axis.set_title(DATASET_LABELS.get(dataset, dataset))
        axis.set_xticks(x)
        axis.set_xticklabels([WORKLOAD_SHORT.get(w, w) for w in workloads], rotation=0, ha="center")
        axis.grid(axis="y", alpha=0.2)

    axes[0].set_ylabel(ylabel)
    fig.suptitle(title)
    fig.text(0.5, 0.03, workload_legend_text(), ha="center", va="center", fontsize=9)
    handles, labels_legend = axes[0].get_legend_handles_labels()
    fig.legend(handles, labels_legend, loc="upper right")
    fig.tight_layout(rect=(0, 0.06, 1, 0.95))
    fig.savefig(output_path, dpi=180)
    plt.close(fig)


def plot_latency_panel(rows: list[dict[str, str]], output_path: Path) -> None:
    width = 0.35

    metrics = [
        ("write_p50_us", "Write p50"),
        ("write_p95_us", "Write p95"),
        ("read_p50_us", "Read p50"),
        ("read_p95_us", "Read p95"),
    ]

    fig, axes = plt.subplots(2, 2, figsize=(16, 10), sharex=True)
    axes_flat = [axes[0][0], axes[0][1], axes[1][0], axes[1][1]]

    for axis, (metric, metric_title) in zip(axes_flat, metrics):
        values = value_map(rows, metric)
        workload_subset = (
            ["wl_write_only", "wl_write_heavy_90_10"]
            if metric.startswith("write_")
            else ["wl_read_only_latest_hit", "wl_read_only_latest_miss"]
        )
        labels = [
            short_combo_label(dataset, workload)
            for dataset in DATASET_ORDER
            for workload in workload_subset
        ]
        points = [
            (dataset, workload)
            for dataset in DATASET_ORDER
            for workload in workload_subset
        ]
        x = list(range(len(points)))

        for idx, strategy in enumerate(["strategy_prefix_scan", "strategy_latest_cf"]):
            xs = [p + (idx - 0.5) * width for p in x]
            ys = [values.get((dataset, workload, strategy)) for dataset, workload in points]
            ys_clean = [float("nan") if y is None else y for y in ys]
            axis.bar(
                xs,
                ys_clean,
                width,
                label=STRATEGY_LABELS[strategy],
                color=STRATEGY_COLORS[strategy],
            )
        axis.set_title(metric_title)
        axis.grid(axis="y", alpha=0.2)
        axis.set_xticks(x)
        axis.set_xticklabels(labels, rotation=0, ha="center", fontsize=8)

    handles, labels_legend = axes[0][0].get_legend_handles_labels()
    fig.legend(handles, labels_legend, loc="upper right")
    fig.suptitle("Settlement Latest Attempt Latency (microseconds)")
    fig.text(0.5, 0.01, workload_legend_text(), ha="center", va="bottom", fontsize=9)
    fig.tight_layout(rect=(0, 0.04, 1, 0.95))
    fig.savefig(output_path, dpi=180)
    plt.close(fig)


def plot_composite(rows: list[dict[str, str]], output_path: Path) -> None:
    key_list = grouped_keys(rows)
    summary = []

    for dataset, workload in key_list:
        key_prefix = (dataset, workload, "strategy_prefix_scan")
        key_latest = (dataset, workload, "strategy_latest_cf")

        row_prefix = next(
            r for r in rows if (r["dataset"], r["workload"], r["strategy"]) == key_prefix
        )
        row_latest = next(
            r for r in rows if (r["dataset"], r["workload"], r["strategy"]) == key_latest
        )

        ops_prefix = parse_float(row_prefix["ops_per_s"])
        ops_latest = parse_float(row_latest["ops_per_s"])
        read_p95_prefix = parse_float(row_prefix["read_p95_us"])
        read_p95_latest = parse_float(row_latest["read_p95_us"])
        size_delta = parse_float(row_latest["db_size_delta_pct"])

        write_penalty = None
        if ops_prefix and ops_latest and ops_prefix > 0:
            write_penalty = ((ops_prefix - ops_latest) / ops_prefix) * 100.0

        speedup = None
        if read_p95_prefix and read_p95_latest and read_p95_latest > 0:
            speedup = read_p95_prefix / read_p95_latest

        summary.append(
            {
                "label": short_combo_label(dataset, workload),
                "write_penalty": write_penalty,
                "read_p95_speedup": speedup,
                "size_delta": size_delta,
            }
        )

    by_dataset = {dataset: [] for dataset in DATASET_ORDER}
    for entry in summary:
        label = entry["label"]
        dataset_key = None
        for d in DATASET_ORDER:
            if label.startswith({
                "ds_1k_realistic_1_25": "1k",
                "ds_10k_realistic_1_25": "10k",
                "ds_50k_realistic_1_25": "50k",
            }[d]):
                dataset_key = d
                break
        if dataset_key is not None:
            by_dataset[dataset_key].append(entry)

    fig, axes = plt.subplots(1, 3, figsize=(20, 6), sharey=False)
    metrics = [
        ("write_penalty", "Write Penalty %", 5.0),
        ("read_p95_speedup", "Read p95 Speedup (x)", None),
        ("size_delta", "DB Size Delta %", None),
    ]

    for axis, (metric_key, title, threshold) in zip(axes, metrics):
        x = list(range(len(WORKLOAD_ORDER)))
        width = 0.22
        for idx, dataset in enumerate(DATASET_ORDER):
            entries = {e["label"].split("-")[1]: e for e in by_dataset[dataset]}
            values = []
            for workload in WORKLOAD_ORDER:
                short = WORKLOAD_SHORT[workload]
                v = entries.get(short, {}).get(metric_key)
                values.append(0.0 if v is None else v)
            xs = [i + (idx - 1) * width for i in x]
            axis.bar(
                xs,
                values,
                width,
                label=DATASET_LABELS[dataset],
                color=DATASET_COLORS[dataset],
            )

        if threshold is not None:
            axis.axhline(threshold, color="#444", linestyle="--", linewidth=1)

        axis.set_title(title)
        axis.set_xticks(x)
        axis.set_xticklabels([WORKLOAD_SHORT[w] for w in WORKLOAD_ORDER], rotation=0)
        axis.grid(axis="y", alpha=0.2)

    fig.suptitle("Settlement Latest Attempt Executive Summary")
    handles, labels_legend = axes[0].get_legend_handles_labels()
    fig.legend(handles, labels_legend, loc="upper right")
    fig.text(0.5, 0.03, workload_legend_text(), ha="center", va="center", fontsize=9)
    fig.tight_layout(rect=(0, 0.06, 1, 0.95))
    fig.savefig(output_path, dpi=180)
    plt.close(fig)


def main() -> None:
    parser = argparse.ArgumentParser(description="Plot settlement latest-attempt benchmark results.")
    parser.add_argument(
        "--csv",
        type=Path,
        required=True,
        help="Path to CSV benchmark results file",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        required=True,
        help="Directory where PNG graphs will be generated",
    )
    args = parser.parse_args()

    rows = load_rows(args.csv)
    args.out_dir.mkdir(parents=True, exist_ok=True)

    plot_grouped_bars(
        rows,
        metric="ops_per_s",
        title="Settlement Latest Attempt Throughput",
        ylabel="ops/s",
        output_path=args.out_dir / "throughput_ops_per_s.png",
    )
    plot_latency_panel(rows, args.out_dir / "latency_panel_us.png")
    plot_grouped_bars(
        rows,
        metric="db_size_mb",
        title="Settlement Latest Attempt DB Size",
        ylabel="MB",
        output_path=args.out_dir / "db_size_mb.png",
    )
    plot_grouped_bars(
        rows,
        metric="db_size_delta_pct",
        title="Settlement Latest Attempt DB Size Delta",
        ylabel="delta %",
        output_path=args.out_dir / "db_size_delta_pct.png",
    )
    plot_composite(rows, args.out_dir / "executive_summary.png")


if __name__ == "__main__":
    main()

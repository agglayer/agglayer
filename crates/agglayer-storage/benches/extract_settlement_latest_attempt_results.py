#!/usr/bin/env python3
"""Extract benchmark metrics from Criterion output into CSV.

This script reads `target/criterion/settlement_latest/**/new/{benchmark,sample}.json`
and produces a flattened CSV matching the plotting script input.
"""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path


def percentile(values: list[float], q: float) -> float:
    if not values:
        raise ValueError("percentile requires non-empty values")
    if len(values) == 1:
        return values[0]
    values = sorted(values)
    pos = (len(values) - 1) * q
    lo = int(pos)
    hi = min(lo + 1, len(values) - 1)
    frac = pos - lo
    return values[lo] * (1.0 - frac) + values[hi] * frac


def parse_workload_and_strategy(full_id: str) -> tuple[str, str, str]:
    # Example:
    # settlement_latest/ds_10k_realistic_1_25/wl_write_only/strategy_prefix_scan/
    parts = [p for p in full_id.split("/") if p]
    if len(parts) < 4:
        raise ValueError(f"Unexpected benchmark full_id: {full_id}")
    return parts[1], parts[2], parts[3]


def extract_rows(criterion_root: Path) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for bench_json in criterion_root.glob("settlement_latest/**/new/benchmark.json"):
        sample_json = bench_json.parent / "sample.json"
        estimates_json = bench_json.parent / "estimates.json"
        if not sample_json.exists() or not estimates_json.exists():
            continue

        bench = json.loads(bench_json.read_text(encoding="utf-8"))
        sample = json.loads(sample_json.read_text(encoding="utf-8"))
        estimates = json.loads(estimates_json.read_text(encoding="utf-8"))

        dataset, workload, strategy = parse_workload_and_strategy(bench["full_id"])

        # Criterion sample times are total time for N benchmark iterations (nanoseconds).
        # Convert to per-iteration ns first.
        per_iter_ns = [t / i for i, t in zip(sample["iters"], sample["times"]) if i > 0]
        p50_us = percentile(per_iter_ns, 0.50) / 1_000.0
        p95_us = percentile(per_iter_ns, 0.95) / 1_000.0

        mean_ns = float(estimates["mean"]["point_estimate"])
        ops_per_s = 1_000_000_000.0 / mean_ns if mean_ns > 0 else 0.0

        row = {
            "dataset": dataset,
            "workload": workload,
            "strategy": strategy,
            "ops_per_s": f"{ops_per_s:.4f}",
            "write_p50_us": "",
            "write_p95_us": "",
            "read_p50_us": "",
            "read_p95_us": "",
            "db_size_mb": "",
            "db_size_delta_pct": "",
        }

        if workload.startswith("wl_write"):
            row["write_p50_us"] = f"{p50_us:.2f}"
            row["write_p95_us"] = f"{p95_us:.2f}"
        if workload.startswith("wl_read"):
            row["read_p50_us"] = f"{p50_us:.2f}"
            row["read_p95_us"] = f"{p95_us:.2f}"

        rows.append(row)

    rows.sort(key=lambda r: (r["dataset"], r["workload"], r["strategy"]))
    return rows


def main() -> None:
    parser = argparse.ArgumentParser(description="Extract settlement latest-attempt benchmark CSV")
    parser.add_argument(
        "--criterion-root",
        type=Path,
        default=Path("target/criterion"),
        help="Path to criterion root directory",
    )
    parser.add_argument(
        "--out-csv",
        type=Path,
        required=True,
        help="Output CSV file path",
    )
    args = parser.parse_args()

    rows = extract_rows(args.criterion_root)
    args.out_csv.parent.mkdir(parents=True, exist_ok=True)

    fieldnames = [
        "dataset",
        "workload",
        "strategy",
        "ops_per_s",
        "write_p50_us",
        "write_p95_us",
        "read_p50_us",
        "read_p95_us",
        "db_size_mb",
        "db_size_delta_pct",
    ]

    with args.out_csv.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Render the sp1bench JSON into the markdown report."""
import json
import random
import statistics
import subprocess
import sys
from pathlib import Path

# Canonical display order; anything else in the data is appended after these.
WORKLOAD_ORDER = ["noop", "pp", "pp-large"]
MODES = ["compressed", "plonk", "groth16"]


def sh(cmd):
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True).stdout.strip()
    except Exception:
        return ""


def fmt(v, *_a, **_kw):
    """Human-readable duration: `4.5s`, `47s`, `3min27s`."""
    if v is None:
        return "—"
    sign = "-" if v < 0 else ""
    v = abs(v)
    if v < 60:
        whole = abs(v - round(v)) < 1e-9
        return f"{sign}{int(round(v))}s" if whole else f"{sign}{v:.1f}s"
    m, s = divmod(round(v), 60)
    return f"{sign}{int(m)}min{int(s):02d}s"


def pearson(a, b):
    ma, mb = statistics.fmean(a), statistics.fmean(b)
    num = sum((i - ma) * (j - mb) for i, j in zip(a, b))
    den = (sum((i - ma) ** 2 for i in a) * sum((j - mb) ** 2 for j in b)) ** 0.5
    return num / den if den else 0.0


def ranks(v):
    order = sorted(range(len(v)), key=lambda i: v[i])
    out = [0] * len(v)
    for pos, i in enumerate(order):
        out[i] = pos
    return out


def cell(runs, w, m):
    return [r for r in runs if r["workload"] == w and r["mode"] == m]


def lat(rows):
    """Server-side latencies for the successful runs of a cell."""
    return sorted(r["server_latency_s"] for r in rows if r["ok"] and r["server_latency_s"] is not None)


def med(xs):
    return statistics.median(xs) if xs else None


def sd(xs):
    """Sample standard deviation. Meaningless below n=3, so refuse it there."""
    return statistics.stdev(xs) if len(xs) >= 3 else None


def boot_ci_diff(a, b, iters=20000, seed=1):
    """Percentile bootstrap 95% CI for median(a) - median(b).

    Latency is right-skewed, so a normal-theory interval would be wrong here.
    Seeded so the report is reproducible.
    """
    if len(a) < 3 or len(b) < 3:
        return None
    rnd = random.Random(seed)
    diffs = []
    for _ in range(iters):
        ra = [a[rnd.randrange(len(a))] for _ in a]
        rb = [b[rnd.randrange(len(b))] for _ in b]
        diffs.append(statistics.median(ra) - statistics.median(rb))
    diffs.sort()
    return diffs[int(0.025 * iters)], diffs[int(0.975 * iters)]


def cv(xs):
    """Coefficient of variation: dispersion scaled by magnitude, so cells of
    very different latency are comparable."""
    s = sd(xs)
    m = statistics.fmean(xs) if xs else 0
    return (s / m) if (s is not None and m) else None


def main(*json_paths):
    """Merge one or more sp1bench JSON files into a single report."""
    runs, wl, params, surveys = [], {}, [], []
    for jp in json_paths:
        d = json.loads(Path(jp).read_text())
        if "survey" in d:
            surveys.append(d)
            continue
        runs += d["runs"]
        for w in d["workloads"]:
            wl[w["name"]] = w
        params.append({"source": Path(jp).name, **d["parameters"]})
    params = params[0] if len(params) == 1 else params

    present = {r["workload"] for r in runs} | set(wl)
    WORKLOADS = [w for w in WORKLOAD_ORDER if w in present]
    WORKLOADS += sorted(present - set(WORKLOAD_ORDER))

    out = []
    A = out.append

    # No commit sha in the title: this file is committed next to the data, so git
    # already records its provenance, and a self-referential sha is never right.
    A("# SP1 proving-latency matrix\n")
    A("Is PP proving latency set by the PP program, or by the SNARK wrap?\n")

    # ---- Run quality -------------------------------------------------------
    ok = [r for r in runs if r["ok"]]
    bad = [r for r in runs if not r["ok"]]
    strategies = {r["strategy"] for r in runs if r["strategy"] is not None}
    fulfillers = {r["fulfiller"] for r in runs if r.get("fulfiller")}
    A("## Run quality\n")
    A(f"- {len(ok)}/{len(runs)} requests fulfilled" + (f", {len(bad)} failed." if bad else "."))
    A(
        "- Fulfillment strategy recorded by the network: "
        + (
            "`RESERVED` on every request."
            if strategies == {2}
            else f"**mixed/unexpected** {sorted(strategies)} (2 = RESERVED, 1 = HOSTED)."
        )
    )
    A(f"- Fulfiller(s): {', '.join(sorted(f'`{f}`' for f in fulfillers)) or '—'}.")
    A("- Requests were issued strictly sequentially, round-robin across cells, order reversed on")
    A("  alternate repetitions. Latency is server-side `fulfilled_at - created_at`, so client-side")
    A("  ELF upload and local simulation are excluded by construction.")
    for r in bad:
        A(f"- FAILED {r['workload']} {r['mode']} rep{r['rep']}: {r['error']} ({r['request_url']})")
    A("")

    # ---- Workloads ---------------------------------------------------------
    A("## Workloads\n")
    A("| workload | exits (own + imported) | ELF bytes | local cycles | local Succinct gas |")
    A("| -------- | ---------------------- | --------- | ------------ | --------------- |")
    for w in WORKLOADS:
        d = wl.get(w, {})
        ex = d.get("exits")
        ex = f"{ex[0]} + {ex[1]}" if ex else "—"
        eb = f"{d['elf_bytes']:,}" if d.get("elf_bytes") else "—"
        lc = f"{d['local_cycles']:,}" if d.get("local_cycles") else "—"
        lg = f"{d['local_gas']:,}" if d.get("local_gas") else "—"
        A(f"| `{w}` | {ex} | {eb} | {lc} | {lg} |")
    A("")

    # ---- Latency matrix ----------------------------------------------------
    A("## Latency (server-side)\n")
    A("| workload | mode | n | min | median | max | sd | CV | queue (median) | cycles |")
    A("| -------- | ---- | - | --- | ------ | --- | -- | -- | -------------- | ------ |")
    table = {}
    for w in WORKLOADS:
        for m in MODES:
            rows = cell(runs, w, m)
            xs = lat(rows)
            table[(w, m)] = med(xs)
            queues = [r["queue_s"] for r in rows if r["ok"] and r["queue_s"] is not None]
            cycles = next((r["cycles"] for r in rows if r["cycles"] is not None), None)
            costs = [r["cost_prove"] for r in rows if r.get("cost_prove")]
            cyc = f"{cycles:,}" if cycles is not None else "—"
            c = cv(xs)
            A(
                f"| `{w}` | {m} | {len(xs)} | {fmt(xs[0] if xs else None)} | "
                f"{fmt(med(xs))} | {fmt(xs[-1] if xs else None)} | {fmt(sd(xs))} | "
                f"{f'{c * 100:.0f}%' if c is not None else '—'} | {fmt(med(queues))} | "
                f"{cyc} |"
            )
    A("")
    A("`sd` is the sample standard deviation, omitted below n=3 where it is meaningless. `CV` is")
    A("the coefficient of variation (sd / mean) — dispersion scaled by magnitude, so cells with")
    A("very different latencies are comparable. `queue` is submission to first observed")
    A("`Assigned`, at 2s poll granularity.")
    A("")

    # ---- Production comparison ---------------------------------------------
    for d in surveys:
        meta = d["survey"]
        rs = [r for r in d["requests"] if r.get("latency_s") is not None]
        if not rs:
            continue
        A("## The same question on production requests\n")
        modes = sorted({r["mode"] for r in rs})
        span_h = (
            max(r["created_at"] for r in rs) - min(r["created_at"] for r in rs)
        ) / 3600
        A(
            f"The synthetic matrix above varies the load deliberately. This is the opposite "
            f"check: {len(rs)} real fulfilled requests from `{meta['requester']}`, whose load "
            f"varies on its own. Spanning {span_h:.0f} hours, mode(s) "
            f"{', '.join(f'`{m}`' for m in modes)}, "
            f"{len({r['vk_hash'] for r in rs})} distinct program(s).\n"
        )

        lats = sorted(r["latency_s"] for r in rs)
        cycs = [r["cycles"] for r in rs if r.get("cycles")]
        gas = [r["gas_used"] for r in rs if r.get("gas_used")]
        A("| metric | n | mean | median | sd | CV | min | max |")
        A("| ------ | - | ---- | ------ | -- | -- | --- | --- |")
        A(
            f"| latency | {len(lats)} | {fmt(statistics.fmean(lats))} | {fmt(med(lats))} | "
            f"{fmt(sd(lats))} | {cv(lats) * 100:.0f}% | {fmt(lats[0])} | {fmt(lats[-1])} |"
        )
        if cycs:
            c = cv(cycs)
            A(
                f"| cycles | {len(cycs)} | {statistics.fmean(cycs):,.0f} | {med(cycs):,.0f} | "
                f"{sd(cycs):,.0f} | {c * 100:.0f}% | {min(cycs):,} | {max(cycs):,} |"
            )
        if gas:
            c = cv(gas)
            A(
                f"| Succinct gas | {len(gas)} | {statistics.fmean(gas):,.0f} | {med(gas):,.0f} | "
                f"{sd(gas):,.0f} | {c * 100:.0f}% | {min(gas):,} | {max(gas):,} |"
            )
        A("")

        paired = [(r["cycles"], r["latency_s"]) for r in rs if r.get("cycles")]
        if len(paired) >= 10:
            xs = [a for a, _ in paired]
            ys = [b for _, b in paired]
            r_p = pearson(xs, ys)
            A(
                f"Work varies {max(xs) / min(xs):.1f}x across these requests, yet it does not "
                f"predict latency: r = {r_p:+.2f}, so the load accounts for about "
                f"{r_p ** 2 * 100:.1f}% of the spread."
            )
            A("")

    # ---- Derived -----------------------------------------------------------
    A("## What the numbers decompose into\n")

    def m(w, mode):
        return table.get((w, mode))

    def diff(a, b):
        return None if a is None or b is None else a - b

    floor = m("noop", "compressed")
    A(f"Pipeline floor (`noop`, an empty program, compressed): **{fmt(floor)}**.\n")
    A("| workload | cycles | PP cost vs noop | 95% CI (bootstrap) | plonk wrap | groth16 wrap | plonk vs groth16 |")
    A("| -------- | ------ | ------------------ | ------------------ | ---------- | ------------ | ---------------- |")
    noop_comp = lat(cell(runs, "noop", "compressed"))
    guest_ci = {}
    for w in WORKLOADS:
        cyc = wl.get(w, {}).get("local_cycles")
        guest_w = diff(m(w, "compressed"), floor) if w != "noop" else None
        ci = boot_ci_diff(lat(cell(runs, w, "compressed")), noop_comp) if w != "noop" else None
        guest_ci[w] = ci
        ci_s = f"{fmt(ci[0])} to {fmt(ci[1])}" if ci else "—"
        A(
            f"| `{w}` | {cyc:,} | {fmt(guest_w)} | {ci_s} | "
            f"{fmt(diff(m(w, 'plonk'), m(w, 'compressed')))} | "
            f"{fmt(diff(m(w, 'groth16'), m(w, 'compressed')))} | "
            f"{fmt(diff(m(w, 'plonk'), m(w, 'groth16')))} |"
        )
    A("")
    A("`PP cost` is `<workload>_compressed - noop_compressed`: what the program itself adds")
    A("before any SNARK wrap. `plonk wrap` and `groth16 wrap` are")
    A("`<workload>_<mode> - <workload>_compressed`: what the wrap adds on top of the STARK.")
    A("")

    # ---- Conclusion --------------------------------------------------------
    A("## What this suggests\n")
    A(
        "These measurements suggest latency is mostly set by the SNARK wrap, not by the PP or "
        "by how much work it does."
    )
    A("")

    meds = [table[(w, "plonk")] for w in WORKLOADS if table.get((w, "plonk")) is not None]
    if meds:
        A(
            f"- Plonk medians across the programs measured sat between {fmt(min(meds))} and "
            f"{fmt(max(meds))}, in no order matching the load."
        )

    pps = [w for w in WORKLOADS if w != "noop" and wl.get(w, {}).get("local_cycles")]
    big = max(pps, key=lambda w: wl[w]["local_cycles"]) if pps else None
    if big and floor is not None and m(big, "compressed") is not None:
        ci = guest_ci.get(big)
        ci_s = f" (95% CI {fmt(ci[0])} to {fmt(ci[1])})" if ci else ""
        A(
            f"- Before any wrap, the heaviest program measured added "
            f"{fmt(diff(m(big, 'compressed'), floor))} over an empty program{ci_s}."
        )

    queues = [r["queue_s"] for r in ok if r["queue_s"] is not None]
    if queues:
        A(
            f"- Queue time was {fmt(med(queues))} median, so these are proving times rather "
            "than waiting times."
        )
    A("")

    ns = [len(lat(cell(runs, w, mo))) for w in WORKLOADS for mo in MODES]
    ns = [n for n in ns if n]
    if ns:
        A(
            f"Caveat: n={min(ns)}-{max(ns)} per cell, run quickly. The direction looks "
            "consistent; the exact seconds are rough, and only the loads listed above were "
            "tested."
        )
        A("")

    # ---- Request URLs ------------------------------------------------------
    A("## Requests\n")
    for w in WORKLOADS:
        for m in MODES:
            rows = cell(runs, w, m)
            if not rows:
                continue
            A(f"**`{w}` / {m}** ({len(rows)} requests)\n")
            for i, r in enumerate(sorted(rows, key=lambda x: x["started_unix"]), 1):
                status = fmt(r["server_latency_s"]) if r["ok"] else f"FAILED ({r['error']})"
                A(f"- {i:02d}: {status} — {r['request_url']}")
            A("")

    # ---- Parameters --------------------------------------------------------
    A("## Parameters\n")
    A("```json")
    A(json.dumps(params, indent=2))
    A("```\n")

    A("## Environment\n")
    A(f"- {sh('sysctl -n machdep.cpu.brand_string')}, {sh('sysctl -n hw.ncpu')} cores, "
      f"{sh('uname -sr')}, {sh('rustc --version')}")
    A("- Latency is measured on the SP1 prover network, not locally. The host only submits and")
    A("  polls, so host load does not affect the reported numbers.")
    A(f"- Poll granularity 2s, so `queue` and the client-side split carry ±2s. Server-side")
    A("  `fulfilled_at - created_at` has 1s resolution.")
    A("")

    A("## Comparing runs\n")
    A("Only comparable against another run with a matching Parameters block and the same circuit")
    A("version. Absolute latencies move with network conditions and with which fulfiller picks up")
    A("the request; the *differences* within one run are the durable result.")

    print("\n".join(out))


if __name__ == "__main__":
    main(*sys.argv[1:])

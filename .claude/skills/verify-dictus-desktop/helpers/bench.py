#!/usr/bin/env python3
"""Tables of bench.sh: samples from the journeys' report.txt, a Markdown summary, a two-run comparison.

    bench.py collect <bench-dir> <run-dir>...     bench.csv (one row per dictation) and bench.md
    bench.py compare <before.csv> <after.csv>     Markdown: medians side by side, delta, verdict

Timings per clip are the warm dictations (seq > 1); the first dictation after launch has its own
table, next to the cold model load. WER uses every dictation.

Only ids and numbers reach the CSV: every field is parsed against a fixed schema (a number, or an id
of [A-Za-z0-9._-]), so no corpus text or transcript can get into the bench files whatever a journey
printed. A line that does not parse is an error, not a skipped row.
"""
import csv
import os
import re
import statistics
import sys

ID = re.compile(r"[A-Za-z0-9._-]+")
NUM = re.compile(r"-?[0-9]+(\.[0-9]+)?|na")
SAMPLE_FIELDS = [
    ("model", ID), ("clip", ID), ("lang", ID), ("category", ID), ("dur_s", NUM), ("rep", NUM),
    ("seq", NUM), ("verdict", ID), ("wer", NUM), ("wer_sp", NUM), ("ref_words", NUM),
    ("hyp_words", NUM), ("recorded_ms", NUM), ("stop_to_text_ms", NUM), ("transcribe_ms", NUM),
    ("cli_stop_ms", NUM), ("load_ms", NUM), ("run", ID),
]
COLUMNS = [name for name, _ in SAMPLE_FIELDS]


def die(msg):
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def checked(row):
    for name, pattern in SAMPLE_FIELDS:
        if not pattern.fullmatch(str(row.get(name, ""))):
            die(f"field {name} of {row.get('clip', '?')} does not parse as {pattern.pattern}")
    return {name: row[name] for name in COLUMNS}


def samples_of(run_dir):
    run, load_ms, rows = os.path.basename(run_dir), None, []
    for line in open(os.path.join(run_dir, "report.txt"), errors="replace"):
        m = re.search(r"JOURNEY dictation load model=(\S+) load_ms=(\d+)$", line)
        if m:
            load_ms = m.group(2)
            continue
        m = re.search(r"JOURNEY dictation (\S+) (PASS|FAIL) (wer=.*)$", line)
        if not m:
            continue
        tokens = m.group(3).split()
        if not all("=" in t for t in tokens):
            die(f"a JOURNEY line of {run} has a field that is not key=value")
        kv = dict(t.split("=", 1) for t in tokens)
        kv.update(clip=m.group(1), verdict=m.group(2), load_ms=load_ms or "na", run=run)
        rows.append(checked(kv))
    return rows


def warm(rows):
    return [r for r in rows if r["seq"] != "1"]


def med_range(values, fmt="{:.0f}"):
    v = sorted(float(x) for x in values if x != "na")
    if not v:
        return "n/a"
    if len(v) == 1:
        return fmt.format(v[0])
    return f"{fmt.format(statistics.median(v))} ({fmt.format(v[0])}–{fmt.format(v[-1])})"


def summary(rows):
    out = ["| Model | Clip | n | WER | WER sp | Stop → text ms (warm) | Transcribe ms (warm) |", "|---|---|---|---|---|---|---|"]
    by = {}
    for r in rows:
        by.setdefault((r["model"], r["clip"]), []).append(r)
    for (model, clip), rs in by.items():
        out.append(f"| {model} | {clip} | {len(rs)} | {med_range([r['wer'] for r in rs], '{:.3f}')} | "
                   f"{med_range([r['wer_sp'] for r in rs], '{:.3f}')} | {med_range([r['stop_to_text_ms'] for r in warm(rs)])} | "
                   f"{med_range([r['transcribe_ms'] for r in warm(rs)])} |")
    out += ["", "| Model | Load ms (cold) | First dictation | Stop → text first | Same clip after, median | "
            "Transcribe first | Same clip after, median |", "|---|---|---|---|---|---|---|"]
    for model in dict.fromkeys(r["model"] for r in rows):
        rs = [r for r in rows if r["model"] == model]
        first = min(rs, key=lambda r: int(r["seq"]))
        later = [r for r in rs if r["clip"] == first["clip"] and r is not first]
        out.append(f"| {model} | {first['load_ms']} | {first['clip']} | {first['stop_to_text_ms']} | "
                   f"{med_range([r['stop_to_text_ms'] for r in later])} | {first['transcribe_ms']} | "
                   f"{med_range([r['transcribe_ms'] for r in later])} |")
    return "\n".join(out)


def cmd_collect(bench_dir, run_dirs):
    rows = [r for d in run_dirs for r in samples_of(d)]
    if not rows:
        die("no JOURNEY dictation sample in the given runs")
    with open(os.path.join(bench_dir, "bench.csv"), "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=COLUMNS)
        w.writeheader()
        w.writerows(rows)
    with open(os.path.join(bench_dir, "bench.md"), "w") as f:
        f.write(summary(rows) + "\n")
    print(f"BENCH TABLES {bench_dir}/bench.csv rows={len(rows)} {bench_dir}/bench.md")


def load_csv(path):
    with open(path, newline="") as f:
        return [checked(r) for r in csv.DictReader(f)]


def cmd_compare(before, after):
    a, b = load_csv(before), load_csv(after)
    print("| Model | Clip | Metric | Before | After | Δ median | Verdict |")
    print("|---|---|---|---|---|---|---|")
    keys = dict.fromkeys((r["model"], r["clip"]) for r in a + b)
    for model, clip in keys:
        ra = [r for r in a if (r["model"], r["clip"]) == (model, clip)]
        rb = [r for r in b if (r["model"], r["clip"]) == (model, clip)]
        if not ra or not rb:
            print(f"| {model} | {clip} | | {'missing' if not ra else len(ra)} | {'missing' if not rb else len(rb)} | | not comparable |")
            continue
        for metric, fmt in (("stop_to_text_ms", "{:.0f}"), ("transcribe_ms", "{:.0f}"), ("wer", "{:.3f}")):
            pick = (lambda rs: rs) if metric == "wer" else warm
            va = [float(r[metric]) for r in pick(ra) if r[metric] != "na"]
            vb = [float(r[metric]) for r in pick(rb) if r[metric] != "na"]
            if not va or not vb:
                print(f"| {model} | {clip} | {metric} | {len(va)} | {len(vb)} | | not comparable |")
                continue
            ma, mb = statistics.median(va), statistics.median(vb)
            delta = f"{(mb - ma) / ma * 100:+.0f} %" if ma else fmt.format(mb - ma)
            overlap = max(va) >= min(vb) and max(vb) >= min(va)
            verdict = "within noise" if overlap else ("lower" if mb < ma else "higher")
            print(f"| {model} | {clip} | {metric} | {med_range(va, fmt)} | {med_range(vb, fmt)} | {delta} | {verdict} |")


def main():
    a = sys.argv[1:]
    if len(a) >= 3 and a[0] == "collect":
        cmd_collect(a[1], a[2:])
    elif len(a) == 3 and a[0] == "compare":
        cmd_compare(a[1], a[2])
    else:
        die("usage: see the docstring")


if __name__ == "__main__":
    main()

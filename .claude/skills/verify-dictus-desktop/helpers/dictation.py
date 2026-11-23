#!/usr/bin/env python3
"""Corpus selection, history polling, WER and the leak check of journey-dictation.sh.

    dictation.py clips <corpus> <clip-id|all-<lang>>        file, id, language, duration (TSV)
    dictation.py last-id <history.db>
    dictation.py wait-row <history.db> <after-id> <t_stop_ns> <secs>
    dictation.py score <corpus> <clip-file> <history.db> <row-id> <wer-max>
    dictation.py transcribe-ms <log-dir> <mark>
    dictation.py leak-check <corpus> <history.db> <run-dir>

The corpus is internal-testing-only: expected texts and transcripts are read into memory and never
printed. Output carries ids, counts, scores and timings only.

WER normalisation (both sides): NFKC, lowercase, a time « 14:30 » / « 14h30 » / « 14 h 30 » becomes
« 14 h 30 », digits and letters are split apart, apostrophes and hyphens become spaces, every
non-word character is dropped; tokens are \\w+ runs. `wer_sp` additionally removes spoken
punctuation (« virgule », « point d'interrogation », …) from both sides, so a clip that dictates
« virgule » scores the same whether the app writes the word or the mark.

Verdict, per clip: the capture gate always applies (the saved recording is at least as long as the
clip, the transcript is not empty). Category `short` is also gated on min(wer, wer_sp) <= wer-max.
Other categories (`long-codeswitch`) report their WER without gating on it: see SKILL.md.
"""
import json
import os
import re
import sqlite3
import sys
import time
import unicodedata

SPOKEN_PUNCTUATION = sorted([
    "point d interrogation", "point d exclamation", "points de suspension", "point virgule",
    "deux points", "à la ligne", "nouvelle ligne", "nouveau paragraphe", "ouvrez les guillemets",
    "fermez les guillemets", "ouvrir les guillemets", "fermer les guillemets", "virgule", "point",
], key=len, reverse=True)
LEAK_NGRAM = 5


def die(msg):
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


def words(text):
    t = unicodedata.normalize("NFKC", text).lower()
    t = re.sub(r"(\d)\s*[h:]\s*(\d)", r"\1 h \2", t)
    t = re.sub(r"(?<=\d)(?=[^\W\d_])|(?<=[^\W\d_])(?=\d)", " ", t)
    t = re.sub(r"[’'`\-‐–]", " ", t)
    return re.findall(r"\w+", t)


def drop_spoken(ws):
    s = " ".join(ws)
    for phrase in SPOKEN_PUNCTUATION:
        s = re.sub(r"(?<!\S)%s(?!\S)" % re.escape(phrase), " ", s)
    return s.split()


def wer(ref, hyp):
    if not ref:
        die("empty reference")
    prev = list(range(len(hyp) + 1))
    for i, r in enumerate(ref, 1):
        cur = [i] + [0] * len(hyp)
        for j, h in enumerate(hyp, 1):
            cur[j] = min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (r != h))
        prev = cur
    return prev[-1] / len(ref)


def manifest(corpus):
    return json.load(open(os.path.join(corpus, "manifest.json")))["clips"]


def clip_id(c):
    return os.path.splitext(os.path.basename(c["file"]))[0]


def db_connect(path):
    return sqlite3.connect(f"file:{path}?mode=ro", uri=True, timeout=1)


def last_id(db):
    if not os.path.exists(db):
        return 0
    with db_connect(db) as con:
        return con.execute("SELECT COALESCE(MAX(id), 0) FROM transcription_history").fetchone()[0]


def cmd_clips(corpus, sel):
    clips = manifest(corpus)
    if sel.startswith("all-"):
        chosen = [c for c in clips if c["language"] == sel[4:]]
    else:
        chosen = [c for c in clips if clip_id(c) == sel]
    if not chosen:
        die(f"no clip matches {sel}")
    for c in chosen:
        print(f"{c['file']}\t{clip_id(c)}\t{c['language']}\t{c['duration_seconds']}\t{c['category']}")


def cmd_wait_row(db, after, t_stop_ns, secs):
    deadline = time.monotonic() + secs
    while time.monotonic() < deadline:
        try:
            with db_connect(db) as con:
                row = con.execute("SELECT id FROM transcription_history WHERE id > ? ORDER BY id LIMIT 1", (after,)).fetchone()
        except sqlite3.OperationalError:
            row = None
        if row:
            print(f"row={row[0]} stop_to_text_ms={(time.time_ns() - t_stop_ns) // 1_000_000}")
            return
        time.sleep(0.01)
    die(f"no new history entry within {secs}s")


GATED_ON_WER = {"short"}


def cmd_score(corpus, clip_file, db, row_id, wer_max):
    clip = next((c for c in manifest(corpus) if c["file"] == clip_file), None)
    if clip is None:
        die(f"{clip_file} not in the manifest")
    ref_text = clip["text"]
    with db_connect(db) as con:
        row = con.execute("SELECT transcription_text, duration_ms FROM transcription_history WHERE id = ?", (row_id,)).fetchone()
    if row is None:
        die(f"no history row {row_id}")
    ref, hyp = words(ref_text), words(row[0])
    w, w_sp = wer(ref, hyp), wer(drop_spoken(ref), drop_spoken(hyp))
    captured = (row[1] or 0) >= clip["duration_seconds"] * 1000 and hyp
    gated = clip["category"] in GATED_ON_WER
    ok = captured and (min(w, w_sp) <= wer_max or not gated)
    gate = f"capture+wer<={wer_max}" if gated else "capture (wer report-only)"
    print(f"{'PASS' if ok else 'FAIL'} wer={w:.3f} wer_sp={w_sp:.3f} ref_words={len(ref)} hyp_words={len(hyp)} "
          f"recorded_ms={row[1]} gate={gate.replace(' ', '_')}")


def cmd_transcribe_ms(log_dir, mark):
    lines = []
    for name in sorted(os.listdir(log_dir)):
        if name.endswith(".log"):
            lines += open(os.path.join(log_dir, name), errors="replace").read().splitlines()
    unit = {"ns": 1e-6, "µs": 1e-3, "us": 1e-3, "ms": 1, "s": 1000}
    for line in lines[int(mark):]:
        m = re.search(r"Transcription completed in ([0-9.]+)(ns|µs|us|ms|s):", line)
        if m:
            print(round(float(m.group(1)) * unit[m.group(2)]))
            return
    print("na")


def cmd_leak_check(corpus, db, run_dir):
    texts = [c["text"] for c in manifest(corpus)]
    if os.path.exists(db):
        with db_connect(db) as con:
            texts += [r[0] for r in con.execute("SELECT transcription_text FROM transcription_history")]
    needles = set()
    for t in texts:
        if not t:
            continue
        ws = words(t)
        if len(ws) <= LEAK_NGRAM:
            if len(" ".join(ws)) >= 12:
                needles.add(" ".join(ws))
        else:
            needles.update(" ".join(ws[i:i + LEAK_NGRAM]) for i in range(len(ws) - LEAK_NGRAM + 1))
    checked = 0
    for root, _, files in os.walk(run_dir):
        for name in files:
            if name.endswith(".png"):
                continue
            path = os.path.join(root, name)
            haystack = " " + " ".join(words(open(path, errors="replace").read())) + " "
            checked += 1
            if any(" " + n + " " in haystack for n in needles):
                print(f"LEAK CHECK FAIL {os.path.relpath(path, run_dir)} contains corpus or transcript text")
                sys.exit(1)
    print(f"LEAK CHECK OK files={checked} texts={len(texts)}")


def main():
    a = sys.argv[1:]
    if not a:
        die("usage: see the docstring")
    cmd, args = a[0], a[1:]
    if cmd == "clips":
        cmd_clips(*args)
    elif cmd == "last-id":
        print(last_id(args[0]))
    elif cmd == "wait-row":
        cmd_wait_row(args[0], int(args[1]), int(args[2]), float(args[3]))
    elif cmd == "score":
        cmd_score(args[0], args[1], args[2], int(args[3]), float(args[4]))
    elif cmd == "transcribe-ms":
        cmd_transcribe_ms(args[0], args[1])
    elif cmd == "leak-check":
        cmd_leak_check(*args)
    else:
        die(f"unknown command {cmd}")


if __name__ == "__main__":
    main()

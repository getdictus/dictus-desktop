#!/usr/bin/python3 -I
"""Paste targets and timings for journey-paste.sh. Prints counts and times only, never text.

  paste.py counter <out>                       run inside an xterm: count what the tty receives
  paste.py count <out>                         "chars=N first_ns=F last_ns=L" from a counter
  paste.py settle <out> <quiet_s> <timeout_s>  wait until the counter got chars and then none for quiet_s
  paste.py wait-log <log-dir> <mark> <regex> <timeout_s>   print time_ns when a log line after mark matches
  paste.py chars <db> <row-id>                 length of a History row's text
  paste.py window <title>                      "<id> WxH+X+Y" of a mapped window with that exact title
"""
import codecs
import os
import re
import sqlite3
import subprocess
import sys
import time


def die(msg):
    sys.exit(f"error: {msg}")


def write_count(out, n, first, last):
    tmp = out + ".tmp"
    with open(tmp, "w") as f:
        f.write(f"chars={n} first_ns={first} last_ns={last}\n")
    os.replace(tmp, out)


def counter(out):
    import tty

    fd = sys.stdin.fileno()
    tty.setraw(fd)
    dec = codecs.getincrementaldecoder("utf-8")("replace")
    n = first = last = 0
    write_count(out, n, first, last)
    while True:
        data = os.read(fd, 65536)
        if not data:
            return
        last = time.time_ns()
        first = first or last
        n += len(dec.decode(data))
        write_count(out, n, first, last)


def read_count(out):
    try:
        return dict(kv.split("=") for kv in open(out).read().split())
    except (OSError, ValueError):
        return {}


def settle(out, quiet_s, timeout_s):
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        c = read_count(out)
        if int(c.get("chars", 0)) > 0 and time.time_ns() - int(c["last_ns"]) > quiet_s * 1e9:
            print(open(out).read().strip())
            return
        time.sleep(0.05)
    die(f"the counter did not settle within {timeout_s}s: {read_count(out)}")


def log_lines(log_dir):
    lines = []
    for name in sorted(os.listdir(log_dir)):
        if name.endswith(".log"):
            lines += open(os.path.join(log_dir, name), errors="replace").read().splitlines()
    return lines


def wait_log(log_dir, mark, regex, timeout_s):
    deadline = time.monotonic() + timeout_s
    pattern = re.compile(regex)
    while time.monotonic() < deadline:
        if any(pattern.search(line) for line in log_lines(log_dir)[mark:]):
            print(time.time_ns())
            return
        time.sleep(0.02)
    die(f"no log line matching /{regex}/ within {timeout_s}s")


def chars(db, row_id):
    with sqlite3.connect(f"file:{db}?mode=ro", uri=True, timeout=1) as con:
        row = con.execute("SELECT transcription_text FROM transcription_history WHERE id = ?", (row_id,)).fetchone()
    if row is None:
        die(f"no history row {row_id}")
    print(len(row[0] or ""))


def window(title):
    out = subprocess.run(["xwininfo", "-root", "-tree"], capture_output=True, text=True, check=True).stdout
    for line in out.splitlines():
        m = re.match(r'\s*(0x[0-9a-f]+) "([^"]*)".*?(\d+x\d+)\+-?\d+\+-?\d+\s+\+(-?\d+)\+(-?\d+)', line)
        if m and m.group(2) == title:
            info = subprocess.run(["xwininfo", "-id", m.group(1)], capture_output=True, text=True).stdout
            if "IsViewable" in info:
                print(f"{m.group(1)} {m.group(3)}+{m.group(4)}+{m.group(5)}")
                return
    die(f"no mapped window titled {title}")


def main():
    a = sys.argv[1:]
    if not a:
        die("usage: see the docstring")
    cmd, args = a[0], a[1:]
    if cmd == "counter":
        counter(args[0])
    elif cmd == "count":
        print(open(args[0]).read().strip())
    elif cmd == "settle":
        settle(args[0], float(args[1]), float(args[2]))
    elif cmd == "wait-log":
        wait_log(args[0], int(args[1]), args[2], float(args[3]))
    elif cmd == "chars":
        chars(args[0], int(args[1]))
    elif cmd == "window":
        window(args[0])
    else:
        die(f"unknown command {cmd}")


if __name__ == "__main__":
    main()

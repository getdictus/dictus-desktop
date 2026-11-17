#!/usr/bin/python3 -I
"""Drive the Dictus window on $DISPLAY through XTEST (ctypes on the system libX11/libXtst).

  x.py window                      print "<id> WxH+X+Y" of the mapped main window ("Dictus" / "Dictus Dev")
  x.py click X Y [--button N] [--abs]   click at X,Y relative to that window (absolute with --abs)
  x.py move X Y [--abs]
  x.py scroll up|down [N] [--at X Y]    N wheel steps (default 3), over X,Y if given
  x.py type TEXT                   type ASCII text with the US keymap of Xvfb
  x.py key NAME[+NAME...]          e.g. Return, Tab, Escape, ctrl+a, shift+Tab
"""
import ctypes
import os
import re
import subprocess
import sys
import time

TITLE = re.compile(os.environ.get("DICTUS_WINDOW_TITLE", r"^Dictus( Dev)?$"))


def windows():
    out = subprocess.run(["xwininfo", "-root", "-tree"], capture_output=True, text=True, check=True).stdout
    for line in out.splitlines():
        m = re.match(r'\s*(0x[0-9a-f]+) "([^"]*)".*?(\d+)x(\d+)\+-?\d+\+-?\d+\s+\+(-?\d+)\+(-?\d+)', line)
        if m and TITLE.match(m.group(2)):
            wid, w, h, x, y = m.group(1), *map(int, m.group(3, 4, 5, 6))
            info = subprocess.run(["xwininfo", "-id", wid], capture_output=True, text=True).stdout
            if "IsViewable" in info and w > 50 and h > 50:
                yield wid, w, h, x, y


def main_window():
    found = sorted(windows(), key=lambda t: t[1] * t[2], reverse=True)
    if not found:
        sys.exit("error: no mapped Dictus window on " + os.environ.get("DISPLAY", "?"))
    return found[0]


x11 = ctypes.CDLL("libX11.so.6")
xtst = ctypes.CDLL("libXtst.so.6")
x11.XOpenDisplay.restype = ctypes.c_void_p
x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
x11.XStringToKeysym.restype = ctypes.c_ulong
x11.XStringToKeysym.argtypes = [ctypes.c_char_p]
x11.XKeysymToKeycode.restype = ctypes.c_ubyte
x11.XKeysymToKeycode.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
x11.XKeycodeToKeysym.restype = ctypes.c_ulong
x11.XKeycodeToKeysym.argtypes = [ctypes.c_void_p, ctypes.c_ubyte, ctypes.c_int]
x11.XFlush.argtypes = [ctypes.c_void_p]
x11.XSync.argtypes = [ctypes.c_void_p, ctypes.c_int]
for fn, args in {
    "XTestFakeMotionEvent": [ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_ulong],
    "XTestFakeButtonEvent": [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong],
    "XTestFakeKeyEvent": [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int, ctypes.c_ulong],
}.items():
    getattr(xtst, fn).argtypes = args

dpy = None


def display():
    global dpy
    if dpy is None:
        dpy = x11.XOpenDisplay(None)
        if not dpy:
            sys.exit("error: cannot open display " + os.environ.get("DISPLAY", "(unset)"))
    return dpy


def flush():
    x11.XSync(display(), 0)
    time.sleep(0.05)


def to_abs(x, y, absolute):
    if absolute:
        return x, y
    _, _, _, wx, wy = main_window()
    return wx + x, wy + y


def move(x, y):
    xtst.XTestFakeMotionEvent(display(), -1, x, y, 0)
    flush()


def button(n):
    xtst.XTestFakeButtonEvent(display(), n, 1, 0)
    flush()
    xtst.XTestFakeButtonEvent(display(), n, 0, 0)
    flush()


def keycode(name):
    sym = x11.XStringToKeysym(name.encode())
    if not sym:
        sys.exit(f"error: unknown keysym {name!r}")
    code = x11.XKeysymToKeycode(display(), sym)
    if not code:
        sys.exit(f"error: no keycode for {name!r}")
    return sym, code


ALIASES = {"ctrl": "Control_L", "shift": "Shift_L", "alt": "Alt_L", "super": "Super_L", "enter": "Return", "esc": "Escape"}


def chord(names):
    press([keycode(ALIASES.get(n.lower(), n))[1] for n in names])


def press(codes):
    for c in codes:
        xtst.XTestFakeKeyEvent(display(), c, 1, 0)
        flush()
    for c in reversed(codes):
        xtst.XTestFakeKeyEvent(display(), c, 0, 0)
        flush()


SPECIAL = {" ": "space", "\n": "Return", "\t": "Tab"}


def type_text(text):
    for ch in text:
        if ch in SPECIAL:
            chord([SPECIAL[ch]])
            continue
        if not (0x20 < ord(ch) < 0x7F):
            sys.exit(f"error: x.py types ASCII only, got {ch!r}")
        sym, code = ord(ch), x11.XKeysymToKeycode(display(), ord(ch))
        if not code:
            sys.exit(f"error: no keycode for {ch!r}")
        shifted = x11.XKeycodeToKeysym(display(), code, 0) != sym
        press([keycode("Shift_L")[1], code] if shifted else [code])


def main(argv):
    if not argv:
        sys.exit(__doc__)
    cmd, rest = argv[0], argv[1:]
    absolute = "--abs" in rest
    rest = [a for a in rest if a != "--abs"]
    if cmd == "window":
        wid, w, h, x, y = main_window()
        print(f"{wid} {w}x{h}+{x}+{y}")
    elif cmd in ("click", "move"):
        btn = 1
        if "--button" in rest:
            i = rest.index("--button")
            btn = int(rest[i + 1])
            del rest[i:i + 2]
        x, y = to_abs(int(rest[0]), int(rest[1]), absolute)
        move(x, y)
        if cmd == "click":
            button(btn)
        print(f"{cmd} {x},{y}" + (f" button={btn}" if cmd == "click" else ""))
    elif cmd == "scroll":
        direction, steps = rest[0], int(rest[1]) if len(rest) > 1 and rest[1].isdigit() else 3
        if "--at" in rest:
            i = rest.index("--at")
            move(*to_abs(int(rest[i + 1]), int(rest[i + 2]), absolute))
        for _ in range(steps):
            button(4 if direction == "up" else 5)
        print(f"scroll {direction} {steps}")
    elif cmd == "type":
        type_text(" ".join(rest))
        print(f"typed {len(' '.join(rest))} chars")
    elif cmd == "key":
        chord(rest[0].split("+"))
        print(f"key {rest[0]}")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])

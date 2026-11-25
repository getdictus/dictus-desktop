#!/usr/bin/python3 -I
"""Drive the Dictus window's title bar under the run's real desktop (launch.sh --wm) and count the hits.

  wm.py ping                                   exit 0 once the desktop answers (gnome-wayland: Eval works)
  wm.py window                                 "<id> WxH+X+Y" of the Dictus frame, like x.py window
  wm.py state                                  minimized, maximized, focus, frame geometry
  wm.py press DX DY                            one hand-paced click at the frame's top-right corner + (DX, DY)
  wm.py clicks minimize|maximize|close DX DY [N]
      N rounds (default 10). Each round clicks once, waits up to 1.5 s for the effect, clicks again if
      nothing happened (up to 4 clicks), then restores the window. Last line:
      WMCLICKS <button> first_click=<hits>/<N> clicks=<clicks per round, 0 = no effect after 4>

Button offsets come from a capture: buttons are right-aligned, so an offset from the top-right corner
survives a move or a restore. Backends, from the run ledger's WM_KIND:
  openbox        X11: WM_STATE, _NET_WM_STATE and the frame window; restore through EWMH messages.
  gnome-wayland  Dictus is a Wayland client of a nested GNOME Shell: state and restore through
                 org.gnome.Shell.Eval on the run's private bus; XTEST on the Xvfb reaches the compositor.
Close hides Dictus to the tray: `cli.sh` (a second instance) shows it again.
"""
import ctypes
import json
import os
import subprocess
import sys
import time

HELPERS = os.path.dirname(os.path.abspath(__file__))
sys.dont_write_bytecode = True
sys.path.insert(0, HELPERS)
import x as xpy  # noqa: E402

TITLE = os.environ.get("DICTUS_WINDOW_TITLE", r"^Dictus( Dev)?$")


def run_dir():
    out = subprocess.run(["bash", "-c", 'source "$1/env.sh" && resolve_run ""', "_", HELPERS],
                         capture_output=True, text=True)
    if out.returncode != 0 or not out.stdout.strip():
        sys.exit("error: no run of this session")
    return out.stdout.strip()


def ledger(run, key):
    value = ""
    with open(os.path.join(run, "run.env")) as f:
        for line in f:
            if line.startswith(key + "="):
                value = line.rstrip("\n").split("=", 1)[1]
    return value


RUN = run_dir()
KIND = ledger(RUN, "WM_KIND")
os.environ["DISPLAY"] = ledger(RUN, "DISPLAY")
if KIND not in ("openbox", "gnome-wayland"):
    sys.exit("error: the run has no --wm desktop (launch.sh --wm openbox|gnome-wayland)")


class GnomeWayland:
    def __init__(self):
        with open(os.path.join(ledger(RUN, "SCRATCH"), "app.env")) as f:
            env = dict(line.rstrip("\n").split("=", 1) for line in f if "=" in line)
        self.env = {k: env[k] for k in ("PATH", "HOME", "XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS")}

    def eval(self, js):
        out = subprocess.run(["gdbus", "call", "--session", "--dest", "org.gnome.Shell", "--object-path",
                              "/org/gnome/Shell", "--method", "org.gnome.Shell.Eval", js],
                             capture_output=True, text=True, env=self.env, timeout=10)
        if out.returncode != 0 or not out.stdout.startswith("(true, "):
            raise RuntimeError(f"Eval failed: {out.stdout.strip()} {out.stderr.strip()}")
        return json.loads(out.stdout.strip()[len("(true, '"):-2].replace("\\'", "'"))

    def find(self):
        return ("global.get_window_actors().map(a=>a.meta_window)"
                f".find(w=>new RegExp({json.dumps(TITLE)}).test(w.get_title()||''))")

    def ping(self):
        return self.eval("global.context.unsafe_mode") is True

    def settle(self):
        """GNOME opens the overview at startup: clicks there pick a window instead of reaching it."""
        self.eval("Main.overview.hide(), 1")
        wait_for(lambda: self.eval("!!Main.overview.visible") is False, 5)

    def state(self):
        s = self.eval(f"(()=>{{const w={self.find()};if(!w)return null;const r=w.get_frame_rect();"
                      "return {id:w.get_id(),min:w.minimized,max:w.get_maximized()===3,focus:w.has_focus(),"
                      "x:r.x,y:r.y,w:r.width,h:r.height}})()")
        if s is None:
            return {"shown": False}
        s["shown"] = not s["min"]
        return s

    def restore(self, button):
        if button == "minimize":
            self.eval(f"(()=>{{const w={self.find()};w.unminimize();w.activate(global.get_current_time());return 1}})()")
        elif button == "maximize":
            self.eval(f"(()=>{{const w={self.find()};w.unmaximize(3);return 1}})()")

    def focus(self):
        self.eval(f"(()=>{{const w={self.find()};if(w)w.activate(global.get_current_time());return 1}})()")


class Openbox:
    def __init__(self):
        x11 = xpy.x11
        u, i, vp = ctypes.c_ulong, ctypes.c_int, ctypes.c_void_p
        x11.XInternAtom.restype = u
        x11.XInternAtom.argtypes = [vp, ctypes.c_char_p, i]
        x11.XDefaultRootWindow.restype = u
        x11.XDefaultRootWindow.argtypes = [vp]
        x11.XQueryTree.argtypes = [vp, u, ctypes.POINTER(u), ctypes.POINTER(u),
                                   ctypes.POINTER(ctypes.POINTER(u)), ctypes.POINTER(ctypes.c_uint)]
        x11.XGetWindowProperty.argtypes = [vp, u, u, ctypes.c_long, ctypes.c_long, i, u, ctypes.POINTER(u),
                                           ctypes.POINTER(i), ctypes.POINTER(u), ctypes.POINTER(u),
                                           ctypes.POINTER(ctypes.POINTER(u))]
        x11.XSendEvent.argtypes = [vp, u, i, ctypes.c_long, vp]
        self.x11, self.client = x11, None

    def atom(self, name):
        return self.x11.XInternAtom(xpy.display(), name.encode(), 0)

    def prop(self, win, name):
        u = ctypes.c_ulong
        typ, fmt, n, after, data = u(), ctypes.c_int(), u(), u(), ctypes.POINTER(u)()
        if self.x11.XGetWindowProperty(xpy.display(), win, self.atom(name), 0, 64, 0, 0, ctypes.byref(typ),
                                       ctypes.byref(fmt), ctypes.byref(n), ctypes.byref(after),
                                       ctypes.byref(data)) != 0 or not data:
            return []
        return [data[k] for k in range(n.value)]

    def frame(self, win):
        d, root = xpy.display(), self.x11.XDefaultRootWindow(xpy.display())
        while True:
            r, parent, kids, n = ctypes.c_ulong(), ctypes.c_ulong(), ctypes.POINTER(ctypes.c_ulong)(), ctypes.c_uint()
            self.x11.XQueryTree(d, win, ctypes.byref(r), ctypes.byref(parent), ctypes.byref(kids), ctypes.byref(n))
            if parent.value in (0, root):
                return win
            win = parent.value

    def settle(self):
        pass

    def ping(self):
        return bool(self.prop(self.x11.XDefaultRootWindow(xpy.display()), "_NET_SUPPORTING_WM_CHECK"))

    def state(self):
        if self.client is None:
            try:
                self.client = int(xpy.main_window()[0], 16)
            except SystemExit:
                return {"shown": False}
        info = subprocess.run(["xwininfo", "-id", hex(self.frame(self.client))], capture_output=True, text=True).stdout
        geo = {k: int(v) for k, v in (
            ("x", info.split("Absolute upper-left X:")[1].split()[0]), ("y", info.split("Absolute upper-left Y:")[1].split()[0]),
            ("w", info.split("Width:")[1].split()[0]), ("h", info.split("Height:")[1].split()[0]))}
        wm_state = self.prop(self.client, "WM_STATE")
        net = set(self.prop(self.client, "_NET_WM_STATE"))
        viewable = "IsViewable" in subprocess.run(["xwininfo", "-id", hex(self.client)], capture_output=True,
                                                   text=True).stdout
        return {"id": hex(self.client), "min": bool(wm_state) and wm_state[0] == 3, "shown": viewable,
                "max": {self.atom("_NET_WM_STATE_MAXIMIZED_VERT"), self.atom("_NET_WM_STATE_MAXIMIZED_HORZ")} <= net,
                "focus": None, **geo}

    def message(self, name, *data):
        class Ev(ctypes.Structure):
            _fields_ = [("type", ctypes.c_int), ("serial", ctypes.c_ulong), ("send_event", ctypes.c_int),
                        ("display", ctypes.c_void_p), ("window", ctypes.c_ulong), ("message_type", ctypes.c_ulong),
                        ("format", ctypes.c_int), ("data", ctypes.c_long * 5), ("pad", ctypes.c_long * 12)]
        ev = Ev(type=33, window=self.client, message_type=self.atom(name), format=32)
        for k, v in enumerate(data):
            ev.data[k] = v
        self.x11.XSendEvent(xpy.display(), self.x11.XDefaultRootWindow(xpy.display()), 0, (1 << 20) | (1 << 19),
                            ctypes.byref(ev))
        xpy.flush()

    def restore(self, button):
        if button == "minimize":
            self.message("_NET_ACTIVE_WINDOW", 2, 0, 0)
        elif button == "maximize":
            self.message("_NET_WM_STATE", 0, self.atom("_NET_WM_STATE_MAXIMIZED_VERT"),
                         self.atom("_NET_WM_STATE_MAXIMIZED_HORZ"), 2)

    def focus(self):
        self.message("_NET_ACTIVE_WINDOW", 2, 0, 0)


desk = GnomeWayland() if KIND == "gnome-wayland" else Openbox()


def wait_for(pred, timeout):
    end = time.time() + timeout
    while time.time() < end:
        if pred():
            return True
        time.sleep(0.05)
    return pred()


def press(dx, dy):
    desk.settle()
    s = desk.state()
    x, y = s["x"] + s["w"] + dx, s["y"] + dy
    xpy.move(x - 30, y + 20)
    time.sleep(0.1)
    xpy.move(x, y)
    time.sleep(0.15)
    xpy.xtst.XTestFakeButtonEvent(xpy.display(), 1, 1, 0)
    xpy.flush()
    time.sleep(0.08)
    xpy.xtst.XTestFakeButtonEvent(xpy.display(), 1, 0, 0)
    xpy.flush()
    return x, y


def reacted(button, before):
    s = desk.state()
    if button == "maximize":
        return s["shown"] and s["max"] != before["max"]
    return not s["shown"]


def restore(button):
    if button == "close":
        subprocess.run([os.path.join(HELPERS, "cli.sh")], check=False, stdout=subprocess.DEVNULL)
    else:
        desk.restore(button)
    if not wait_for(lambda: (s := desk.state())["shown"] and not s["max"], 10):
        sys.exit(f"error: could not restore the window after {button}: {desk.state()}")
    desk.focus()
    time.sleep(0.8)


def clicks(button, dx, dy, rounds):
    per_round = []
    for i in range(rounds):
        before = desk.state()
        n = 0
        while n < 4:
            n += 1
            x, y = press(dx, dy)
            if wait_for(lambda: reacted(button, before), 1.5):
                break
        else:
            n = 0
        per_round.append(n)
        print(f"round {i + 1} {button} at {x},{y}: "
              + (f"reacted after {n} click(s)" if n else "no reaction after 4 clicks"), flush=True)
        restore(button)
    hits = per_round.count(1)
    print(f"WMCLICKS {button} first_click={hits}/{rounds} clicks={','.join(map(str, per_round))}")
    return hits == rounds


def main(argv):
    cmd, rest = (argv[0], argv[1:]) if argv else ("", [])
    if cmd == "ping":
        try:
            sys.exit(0 if desk.ping() else 1)
        except Exception as e:  # noqa: BLE001  (the compositor is still starting)
            sys.exit(f"not ready: {e}")
    elif cmd == "window":
        desk.settle()
        s = desk.state()
        if not s["shown"]:
            sys.exit("error: no shown Dictus window")
        print(f"{s['id']} {s['w']}x{s['h']}+{s['x']}+{s['y']}")
    elif cmd == "state":
        print(json.dumps(desk.state()))
    elif cmd == "press":
        print("press %d,%d" % press(int(rest[0]), int(rest[1])))
    elif cmd == "clicks" and rest and rest[0] in ("minimize", "maximize", "close"):
        sys.exit(0 if clicks(rest[0], int(rest[1]), int(rest[2]), int(rest[3]) if len(rest) > 3 else 10) else 1)
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main(sys.argv[1:])

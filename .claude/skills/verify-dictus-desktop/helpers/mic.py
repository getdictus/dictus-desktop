#!/usr/bin/env python3
"""Virtual microphone of a tier-1 run: a private PipeWire in the run's scratch dir, no session manager.

    mic.py <scratch> conf          print the daemon config (launch.sh writes it to <scratch>/pipewire.conf)
    mic.py <scratch> check         the private graph has the virtual source and no hardware device
    mic.py <scratch> link [secs]   connect the virtual source to the app's capture stream
    mic.py <scratch> play <wav>    write a 16 kHz mono s16 WAV into the source in real time

The daemon loads no ALSA/Bluetooth/V4L2 SPA plugin, so it cannot open a sound card. Without a session
manager nothing configures or links ports, so `link` does what WirePlumber would: PortConfig (dsp) on
both nodes, then pw-link. Every pw-* call runs with the run's HOME and XDG_RUNTIME_DIR only, so it can
only ever reach the private daemon, never the machine's own PipeWire.
"""
import json
import os
import subprocess
import sys
import time
import wave

SOURCE = "dictus-virtual-mic"
RATE = 16000
DSP_FORMAT = "format: { mediaType: audio, mediaSubtype: raw, format: F32P, rate: 48000, channels: 1, position: [ MONO ] }"

CONF = """context.properties = {
  core.daemon = true
  core.name = pipewire-0
  default.clock.rate = 48000
  default.clock.allowed-rates = [ 48000 ]
  support.dbus = false
  mem.allow-mlock = false
}
context.spa-libs = {
  audio.convert.* = audioconvert/libspa-audioconvert
  support.*       = support/libspa-support
}
context.modules = [
  { name = libpipewire-module-protocol-native }
  { name = libpipewire-module-access }
  { name = libpipewire-module-metadata }
  { name = libpipewire-module-spa-node-factory }
  { name = libpipewire-module-client-node }
  { name = libpipewire-module-adapter }
  { name = libpipewire-module-link-factory }
  { name = libpipewire-module-pipe-tunnel
    args = {
      tunnel.mode = source
      pipe.filename = "%(fifo)s"
      audio.format = S16LE
      audio.rate = %(rate)d
      audio.channels = 1
      audio.position = [ MONO ]
      stream.props = { node.name = %(source)s node.description = "Dictus verify virtual mic" }
    }
  }
]
context.objects = [
  { factory = spa-node-factory
    args = { factory.name = support.node.driver node.name = Dummy-Driver node.group = pipewire.dummy priority.driver = 20000 } }
]
"""


def die(msg):
    print(f"error: {msg}", file=sys.stderr)
    sys.exit(1)


class Graph:
    def __init__(self, scratch):
        self.env = {
            "PATH": "/usr/bin:/bin",
            "HOME": os.path.join(scratch, "home"),
            "XDG_RUNTIME_DIR": os.path.join(scratch, "runtime"),
        }

    def run(self, *args, check=True):
        p = subprocess.run(args, env=self.env, capture_output=True, text=True, timeout=10)
        if check and p.returncode != 0:
            die(f"{' '.join(args[:2])} exit {p.returncode}: {p.stderr.strip()[:200]}")
        return p

    def objects(self):
        return json.loads(self.run("pw-dump").stdout or "[]")

    def nodes(self, media_class=None):
        out = []
        for o in self.objects():
            if o.get("type") != "PipeWire:Interface:Node":
                continue
            props = (o.get("info") or {}).get("props") or {}
            if media_class is None or props.get("media.class") == media_class:
                out.append((o["id"], props.get("node.name", "")))
        return out

    def ports(self, node_id, direction):
        res = []
        for o in self.objects():
            if o.get("type") != "PipeWire:Interface:Port":
                continue
            info = o.get("info") or {}
            props = info.get("props") or {}
            if str(props.get("node.id")) == str(node_id) and info.get("direction") == direction:
                res.append(o["id"])
        return res

    def port_config(self, node_id, direction):
        self.run("pw-cli", "set-param", str(node_id), "PortConfig",
                 "{ direction: %s, mode: dsp, monitor: false, control: false, %s }" % (direction, DSP_FORMAT))

    def source_id(self):
        ids = [i for i, name in self.nodes("Audio/Source") if name == SOURCE]
        return ids[0] if ids else None


def check(g):
    devices = [o for o in g.objects() if o.get("type") == "PipeWire:Interface:Device"]
    src = g.source_id()
    if devices:
        die(f"{len(devices)} device(s) in the private graph: it must have none")
    if src is None:
        die(f"no {SOURCE} node in the private graph")
    if not g.ports(src, "output"):
        g.port_config(src, "Output")
        for _ in range(20):
            if g.ports(src, "output"):
                break
            time.sleep(0.1)
    print(f"MIC OK source={src} ports={len(g.ports(src, 'output'))} devices=0")


def link(g, secs):
    src = g.source_id()
    if src is None:
        die(f"no {SOURCE} node")
    deadline = time.monotonic() + secs
    while True:
        streams = g.nodes("Stream/Input/Audio")
        if streams:
            break
        if time.monotonic() > deadline:
            die(f"no capture stream appeared within {secs}s (the app did not open the microphone)")
        time.sleep(0.05)
    if len(streams) > 1:
        die(f"{len(streams)} capture streams, expected the app's only: {streams}")
    app_id, app_name = streams[0]
    if not g.ports(app_id, "input"):
        g.port_config(app_id, "Input")
        while not g.ports(app_id, "input"):
            if time.monotonic() > deadline:
                die(f"capture stream {app_name} got no input ports")
            time.sleep(0.05)
    made = 0
    for out_port in g.ports(src, "output"):
        for in_port in g.ports(app_id, "input"):
            p = g.run("pw-link", str(out_port), str(in_port), check=False)
            if p.returncode == 0:
                made += 1
            elif "File exists" not in p.stderr:
                die(f"pw-link {out_port} {in_port}: {p.stderr.strip()[:200]}")
    print(f"MIC LINKED {SOURCE} -> {app_name} (node {app_id}, new links {made})")


def play(fifo, path):
    with wave.open(path) as w:
        if (w.getframerate(), w.getnchannels(), w.getsampwidth()) != (RATE, 1, 2):
            die(f"{os.path.basename(path)}: need 16 kHz mono s16, got {w.getframerate()} Hz x{w.getnchannels()} {8 * w.getsampwidth()} bit")
        pcm = w.readframes(w.getnframes())
    chunk = RATE * 2 // 50
    ahead = 0.1
    fd = os.open(fifo, os.O_WRONLY)
    t0 = time.monotonic()
    sent = 0
    try:
        while sent < len(pcm):
            n = os.write(fd, pcm[sent:sent + chunk])
            sent += n
            delay = t0 + sent / (RATE * 2) - ahead - time.monotonic()
            if delay > 0:
                time.sleep(delay)
    finally:
        os.close(fd)
    # What is still in the pipe drains at real time; wait for it before the caller stops the recording.
    time.sleep(ahead + 0.25)
    print(f"MIC PLAYED {os.path.basename(path)} {len(pcm) / (RATE * 2):.2f}s in {time.monotonic() - t0:.2f}s")


def main():
    if len(sys.argv) < 3:
        die(__doc__.strip().splitlines()[2])
    scratch, cmd = sys.argv[1], sys.argv[2]
    fifo = os.path.join(scratch, "mic.fifo")
    if cmd == "conf":
        print(CONF % {"fifo": fifo, "rate": RATE, "source": SOURCE})
    elif cmd == "check":
        check(Graph(scratch))
    elif cmd == "link":
        link(Graph(scratch), float(sys.argv[3]) if len(sys.argv) > 3 else 10)
    elif cmd == "play":
        play(fifo, sys.argv[3])
    else:
        die(f"unknown command {cmd}")


if __name__ == "__main__":
    main()

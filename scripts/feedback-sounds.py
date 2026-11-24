#!/usr/bin/env python3
"""Measure and post-process the recording sounds in src-tauri/resources/sounds.

    feedback-sounds.py measure <dir>             one line per <pack>/<sound>.wav
    feedback-sounds.py process <src> <dst>       the Bois, Synthé and Minimal files of <src>, processed into <dst>

`process` takes the files as they were picked (git show 880a552:src-tauri/resources/sounds/...) and
applies, per file: a 2-pole high-pass at HIGHPASS_HZ, a raised-cosine fade-in of FADE_IN_S, then one gain
per file. Every file is brought to the same loudness, the loudest every file reaches with its peak at
PEAK_MAX_DBFS, except a pack that cannot reach it within that peak (Minimal, a tick with a high crest
factor): its two files share the loudest level both reach. Length, sample rate and pitch are unchanged.
Needs ffmpeg. Standard library only.

Loudness is BS.1770 K-weighted, ungated, over the whole sound: the sounds are shorter than the 3 s
short-term window, so this is the short-term loudness of the sound alone.
"""
import array
import math
import os
import subprocess
import sys
import tempfile
import wave

PACKS = ("bois", "synthe", "minimal")
SOUNDS = ("start", "end")
HIGHPASS_HZ = 130
FADE_IN_S = 0.006
PEAK_MAX_DBFS = -16.0
# Packs the common level is computed from; the others are capped by their peak.
REFERENCE_PACKS = ("bois", "synthe")


def read(path):
    with wave.open(path) as w:
        assert w.getsampwidth() == 2, f"{path}: 16-bit expected"
        rate, channels = w.getframerate(), w.getnchannels()
        data = array.array("h", w.readframes(w.getnframes()))
    if sys.byteorder == "big":
        data.byteswap()
    return rate, channels, [s / 32768.0 for s in data]


def write(path, rate, channels, samples):
    data = array.array("h", (max(-32768, min(32767, round(s * 32768.0))) for s in samples))
    if sys.byteorder == "big":
        data.byteswap()
    with wave.open(path, "wb") as w:
        w.setnchannels(channels)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(data.tobytes())


def biquad(x, b, a):
    y, x1, x2, y1, y2 = [], 0.0, 0.0, 0.0, 0.0
    for x0 in x:
        y0 = b[0] * x0 + b[1] * x1 + b[2] * x2 - a[1] * y1 - a[2] * y2
        y.append(y0)
        x1, x2, y1, y2 = x0, x1, y0, y1
    return y


def k_weighting(rate):
    k = math.tan(math.pi * 1681.974450955533 / rate)
    q = 0.7071752369554196
    vh = 10 ** (3.999843853973347 / 20)
    vb = vh ** 0.4996667741545416
    a0 = 1 + k / q + k * k
    shelf = (
        [(vh + vb * k / q + k * k) / a0, 2 * (k * k - vh) / a0, (vh - vb * k / q + k * k) / a0],
        [1.0, 2 * (k * k - 1) / a0, (1 - k / q + k * k) / a0],
    )
    k = math.tan(math.pi * 38.13547087602444 / rate)
    q = 0.5003270373238773
    a0 = 1 + k / q + k * k
    highpass = ([1.0, -2.0, 1.0], [1.0, 2 * (k * k - 1) / a0, (1 - k / q + k * k) / a0])
    return shelf, highpass


def db(x):
    return 20 * math.log10(x) if x > 0 else float("-inf")


def measure(rate, channels, samples):
    shelf, highpass = k_weighting(rate)
    power = 0.0
    for c in range(channels):
        weighted = biquad(biquad(samples[c::channels], *shelf), *highpass)
        power += sum(v * v for v in weighted) / len(weighted)
    first_ms = samples[: channels * rate // 1000]
    return {
        "peak": db(max(abs(s) for s in samples)),
        "rms": db(math.sqrt(sum(s * s for s in samples) / len(samples))),
        "lufs": -0.691 + 10 * math.log10(power),
        "first_ms": max(abs(s) for s in first_ms),
        "length": len(samples) / channels / rate,
    }


def files(root, packs=None):
    for pack in packs or sorted(os.listdir(root)):
        for sound in SOUNDS:
            path = os.path.join(root, pack, f"{sound}.wav")
            if os.path.isfile(path):
                yield f"{pack}/{sound}", path


def cmd_measure(root):
    print(f"{'file':<16}{'length s':>9}{'peak dBFS':>11}{'RMS dBFS':>10}{'LUFS':>8}{'peak 1st ms':>13}")
    for name, path in files(root):
        m = measure(*read(path))
        print(f"{name:<16}{m['length']:>9.3f}{m['peak']:>11.2f}{m['rms']:>10.2f}{m['lufs']:>8.2f}{m['first_ms']:>13.4f}")


def filtered(path, tmp):
    out = os.path.join(tmp, "f.wav")
    subprocess.run(
        ["ffmpeg", "-v", "error", "-y", "-i", path, "-af",
         f"highpass=f={HIGHPASS_HZ}:p=2,afade=t=in:st=0:d={FADE_IN_S}:curve=hsin",
         "-c:a", "pcm_s16le", out],
        check=True,
    )
    return read(out)


def cmd_process(src, dst):
    with tempfile.TemporaryDirectory() as tmp:
        sounds = {name: filtered(path, tmp) for name, path in files(src, PACKS)}
    stats = {name: measure(*s) for name, s in sounds.items()}

    def reachable(names):
        return min(stats[n]["lufs"] + PEAK_MAX_DBFS - stats[n]["peak"] for n in names)

    target = reachable([n for n in stats if n.split("/")[0] in REFERENCE_PACKS])
    for pack in PACKS:
        names = [n for n in stats if n.split("/")[0] == pack]
        level = min(target, reachable(names))
        for name in names:
            rate, channels, samples = sounds[name]
            gain_db = level - stats[name]["lufs"]
            gain = 10 ** (gain_db / 20)
            os.makedirs(os.path.join(dst, pack), exist_ok=True)
            write(os.path.join(dst, f"{name}.wav"), rate, channels, [s * gain for s in samples])
            print(f"{name:<16} gain {gain_db:+.2f} dB -> {level:.2f} LUFS")
    print(f"common level {target:.2f} LUFS, peaks <= {PEAK_MAX_DBFS} dBFS")


def main():
    if len(sys.argv) == 3 and sys.argv[1] == "measure":
        cmd_measure(sys.argv[2])
    elif len(sys.argv) == 4 and sys.argv[1] == "process":
        cmd_process(sys.argv[2], sys.argv[3])
    else:
        sys.exit(__doc__.strip().splitlines()[1])


if __name__ == "__main__":
    main()

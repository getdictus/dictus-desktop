#!/usr/bin/python3 -I
"""Run a PowerShell command in the dictus-win VM through the QEMU guest agent (runs as SYSTEM).

  vm-exec.py <domain> '<powershell>' [timeout-s]

Prints the guest's stdout/stderr, then "[exit N]". Exits with the guest's exit code, 124 on timeout.
Refuses any domain other than dictus-win. From Pierre's vm/scripts/gexec.py.
"""
import base64
import json
import subprocess
import sys
import time

if len(sys.argv) < 3:
    sys.exit(__doc__)
dom, cmd = sys.argv[1], sys.argv[2]
timeout = int(sys.argv[3]) if len(sys.argv) > 3 else 600
if dom != "dictus-win":
    sys.exit(f"error: refusing domain {dom!r}: this skill drives dictus-win only")


def qga(obj):
    out = subprocess.run(["virsh", "-c", "qemu:///system", "qemu-agent-command", dom, json.dumps(obj)],
                         capture_output=True, text=True, check=True).stdout
    return json.loads(out)["return"]


pid = qga({"execute": "guest-exec", "arguments": {
    "path": "powershell.exe",
    "arg": ["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", cmd],
    "capture-output": True}})["pid"]
end = time.time() + timeout
while time.time() < end:
    st = qga({"execute": "guest-exec-status", "arguments": {"pid": pid}})
    if st.get("exited"):
        for k in ("out-data", "err-data"):
            if k in st:
                sys.stdout.write(base64.b64decode(st[k]).decode("utf-8", "replace"))
        code = st.get("exitcode", 1)
        print(f"\n[exit {code}]")
        sys.exit(code)
    time.sleep(3)
print("[timeout]")
sys.exit(124)

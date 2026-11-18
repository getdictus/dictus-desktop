#!/usr/bin/env bash
# Tier 2: the Windows VM dictus-win (libvirt, 4 GB) on this Linux host. Never the domain `windows`.
#   vm.sh start [snapshot]   new tier-2 run, lock + watchdog, revert to <snapshot> (default
#                            clean-no-vcredist), start, wait for guest-ping (max 10 min). Ends: VM READY
#   vm.sh fetch <pr>         download the Windows x64 installer of the latest successful
#                            pr-test-build.yml run for <pr> into the run's scratch dir. Ends: INSTALLER <file>
#   vm.sh serve              serve that dir over HTTP on 192.168.122.1 only. Ends: SERVING <url>
#   vm.sh install            as user dictus (scheduled task, interactive): silent NSIS install. Ends: INSTALLED
#   vm.sh launch             as user dictus: start Dictus, wait 30 s, list its process. Ends: RUNNING pid=…
#   vm.sh ps '<powershell>'  run PowerShell in the guest as SYSTEM (vm-exec.py)
# Screenshot: shot.sh <name> (virsh screenshot of the whole VM screen). Teardown: cleanup.sh
# (destroy + revert to clean-no-vcredist, stop the HTTP server, delete the installers, release the lock).
set -euo pipefail
source "$(dirname "$0")/env.sh"
[ "$DICTUS_VM" = dictus-win ] || die "this skill drives dictus-win only"
PORT="${DICTUS_HTTP_PORT:-8765}"
TASK_USER="dictus"
INSTALL_DIR='C:\Users\dictus\AppData\Local\Dictus'
cmd="${1:?usage: vm.sh start|fetch|serve|install|launch|ps}"; shift
gexec() { "$DICTUS_HELPERS/vm-exec.py" "$DICTUS_VM" "$@"; }

# Run a command line in the guest as user dictus on its desktop: a guest-agent process runs as
# SYSTEM, has no desktop, and a silent per-user NSIS install as SYSTEM lands in the system profile.
as_user() {
  local name="$1" exe="$2" args="${3:+ -Argument '$3'}"
  gexec "\$a = New-ScheduledTaskAction -Execute '$exe'$args;
\$p = New-ScheduledTaskPrincipal -UserId '$TASK_USER' -LogonType Interactive;
Register-ScheduledTask -TaskName '$name' -Action \$a -Principal \$p -Force | Out-Null;
Start-ScheduledTask -TaskName '$name'; 'started $name'" 60
}

case "$cmd" in
  start)
    snap="${1:-$DICTUS_VM_CLEAN_SNAPSHOT}"
    "${VIRSH[@]}" domstate "$DICTUS_VM" | grep -q 'shut off' || die "$DICTUS_VM is not shut off: someone else is using it"
    run="$(run_begin tier2 "vm-$snap" | tail -1)"
    trap 'rc=$?; [ $rc -eq 0 ] || { echo "VM START FAILED: cleaning up" >&2; bash "$DICTUS_HELPERS/cleanup.sh" "$run" >&2 || true; }' EXIT
    run_hold "$run"
    ledger_set "$run" VM_STARTED 1
    ledger_set "$run" SCRATCH "$(mktemp -d /tmp/verify-dictus-desktop.XXXXXX)"
    "${VIRSH[@]}" snapshot-revert "$DICTUS_VM" "$snap" > /dev/null
    "${VIRSH[@]}" domstate "$DICTUS_VM" | grep -q running || "${VIRSH[@]}" start "$DICTUS_VM" > /dev/null
    t0="$(date +%s)"
    until "${VIRSH[@]}" qemu-agent-command "$DICTUS_VM" '{"execute":"guest-ping"}' > /dev/null 2>&1; do
      [ $(($(date +%s) - t0)) -lt 600 ] || die "no guest-ping after 10 min"
      sleep 10
    done
    report "$run" "VM READY snapshot=$snap after $(($(date +%s) - t0))s"
    echo "VM READY run=$run snapshot=$snap boot=$(($(date +%s) - t0))s"
    ;;
  fetch)
    pr="${1:?usage: vm.sh fetch <pr>}"
    run="$(resolve_run)" || die "no run: vm.sh start first"
    name="handy-pr-$pr-x86_64-pc-windows-msvc"
    id=""
    for r in $(gh run list -R getdictus/dictus-desktop --workflow pr-test-build.yml --status success -L 30 --json databaseId --jq '.[].databaseId'); do
      gh api "repos/getdictus/dictus-desktop/actions/runs/$r/artifacts" --jq '.artifacts[] | select(.expired | not) | .name' | grep -qx "$name" && { id="$r"; break; }
    done
    [ -n "$id" ] || die "no successful pr-test-build.yml run with artifact $name: dispatch the workflow for PR #$pr first"
    dir="$(ledger_get "$run" SCRATCH)/installers"
    mkdir -p "$dir"
    gh run download "$id" -R getdictus/dictus-desktop -n "$name" -D "$dir"
    exe="$(find "$dir" -name '*-setup.exe' | head -1)"
    [ -n "$exe" ] || die "no *-setup.exe in $name"
    ledger_set "$run" INSTALLER "$exe"
    report "$run" "FETCHED run=$id artifact=$name file=$(basename "$exe") sha=$(sha256sum "$exe" | cut -c1-16)"
    echo "INSTALLER $exe (workflow run $id)"
    ;;
  serve)
    run="$(resolve_run)" || die "no run: vm.sh start first"
    exe="$(ledger_get "$run" INSTALLER)"; [ -n "$exe" ] || die "vm.sh fetch first"
    setsid python3 -I -m http.server "$PORT" --bind "$DICTUS_HTTP_HOST" --directory "$(dirname "$exe")" \
      < /dev/null > "$run/http.log" 2>&1 &
    ledger_set "$run" HTTP "$(pid_ref $!)"
    sleep 1
    ref_alive "$(ledger_get "$run" HTTP)" || die "HTTP server did not start: see $run/http.log"
    echo "SERVING http://$DICTUS_HTTP_HOST:$PORT/$(basename "$exe")"
    ;;
  install)
    run="$(resolve_run)" || die "no run"
    exe="$(basename "$(ledger_get "$run" INSTALLER)")"
    gexec "Invoke-WebRequest -UseBasicParsing 'http://$DICTUS_HTTP_HOST:$PORT/$exe' -OutFile 'C:\\Users\\Public\\dictus-setup.exe'; (Get-Item 'C:\\Users\\Public\\dictus-setup.exe').Length" 300
    as_user DictusVerifyInstall 'C:\Users\Public\dictus-setup.exe' '/S'
    t0="$(date +%s)"
    until gexec "if (Test-Path '$INSTALL_DIR\\dictus.exe') { exit 0 } else { exit 1 }" 30 > /dev/null 2>&1; do
      [ $(($(date +%s) - t0)) -lt 300 ] || die "no $INSTALL_DIR\\dictus.exe after 5 min"
      sleep 10
    done
    report "$run" "INSTALLED in $(($(date +%s) - t0))s"
    echo "INSTALLED $INSTALL_DIR\\dictus.exe"
    ;;
  launch)
    run="$(resolve_run)" || die "no run"
    as_user DictusVerifyLaunch "$INSTALL_DIR\\dictus.exe"
    sleep 30
    out="$(gexec "Get-Process dictus -ErrorAction SilentlyContinue | Select-Object Id,SessionId,MainWindowTitle | Format-Table -HideTableHeaders" 60 || true)"
    echo "$out" > "$run/vm-process.txt"
    pid="$(awk 'NF >= 2 && $1 ~ /^[0-9]+$/ {print $1; exit}' <<< "$out")"
    [ -n "$pid" ] || die "dictus.exe is not running 30 s after launch: see $run/vm-process.txt"
    report "$run" "RUNNING pid=$pid"
    echo "RUNNING pid=$pid ($run/vm-process.txt)"
    ;;
  ps) gexec "$1" "${2:-600}" ;;
  *) die "unknown command $cmd" ;;
esac

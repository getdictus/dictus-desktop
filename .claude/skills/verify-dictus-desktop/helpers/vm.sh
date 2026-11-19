#!/usr/bin/env bash
# Tier 2: the Windows VM dictus-win (libvirt, 4 GB) on this Linux host. Never the domain `windows`.
#   vm.sh start [snapshot]   new tier-2 run, lock + watchdog, revert to <snapshot> (default
#                            clean-no-vcredist), start, wait for guest-ping (max 10 min). Ends: VM READY
#   vm.sh fetch <pr>         download the Windows x64 installer of the latest successful
#                            pr-test-build.yml run for <pr> into the run's scratch dir. Ends: INSTALLER <file>
#   vm.sh fetch-release <release-id> [nsis|msi]
#                            download the Windows x64 installer (default NSIS) of a release, drafts
#                            included, by asset id through the API into the same dir. Ends: INSTALLER <file>
#   vm.sh serve              serve that dir over HTTP on 192.168.122.1 only. Ends: SERVING <url>
#   vm.sh install            NSIS: silent per-user install as user dictus (scheduled task), wait for the
#                            task to finish with result 0. MSI: per-machine msiexec /qn as SYSTEM.
#                            Ends: INSTALLED <dictus.exe>
#   vm.sh launch             as user dictus: start Dictus, wait 30 s, list its process. Ends: RUNNING pid=…
#                            or an error with the launch task's state and result
#   vm.sh ps '<powershell>'  run PowerShell in the guest as SYSTEM (vm-exec.py)
# Screenshot: shot.sh <name> (virsh screenshot of the whole VM screen). Teardown: cleanup.sh
# (destroy + revert to clean-no-vcredist, stop the HTTP server, delete the installers, release the lock).
set -euo pipefail
source "$(dirname "$0")/env.sh"
[ "$DICTUS_VM" = dictus-win ] || die "this skill drives dictus-win only"
PORT="${DICTUS_HTTP_PORT:-8765}"
TASK_USER="dictus"
NSIS_EXE='C:\Users\dictus\AppData\Local\Dictus\dictus.exe'
MSI_EXE='C:\Program Files\Dictus\dictus.exe'
REPO="getdictus/dictus-desktop"
cmd="${1:?usage: vm.sh start|fetch|fetch-release|serve|install|launch|ps}"; shift
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

# "<State> 0x<LastTaskResult>" of a scheduled task. 0x00041303: has not run yet; 0x00041301: running.
task_state() {
  gexec "\$t = Get-ScheduledTask -TaskName '$1'; '{0} 0x{1:X8}' -f \$t.State, (\$t | Get-ScheduledTaskInfo).LastTaskResult" 60 |
    tr -d '\r' | grep -E '^[A-Za-z]+ 0x[0-9A-F]{8}' | tail -1 || true
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
  fetch-release)
    rel="${1:?usage: vm.sh fetch-release <release-id> [nsis|msi]}"
    case "${2:-nsis}" in
      nsis) re='_x64-setup\.exe$' ;;
      msi) re='_x64_.*\.msi$' ;;
      *) die "installer type is nsis or msi, not $2" ;;
    esac
    run="$(resolve_run)" || die "no run: vm.sh start first"
    assets="$(gh api "repos/$REPO/releases/$rel/assets" --paginate --jq '.[] | "\(.id) \(.size) \(.name)"')" ||
      die "release $rel not readable: check the id (gh api repos/$REPO/releases)"
    match="$(awk -v re="$re" '$3 ~ re' <<< "$assets")"
    [ "$(grep -c . <<< "$match")" = 1 ] || die "expected one asset matching $re in release $rel, found: ${match:-none}"
    read -r aid size name <<< "$match"
    dir="$(ledger_get "$run" SCRATCH)/installers"
    mkdir -p "$dir"
    gh api -H 'Accept: application/octet-stream' "repos/$REPO/releases/assets/$aid" > "$dir/$name"
    [ "$(stat -c %s "$dir/$name")" = "$size" ] || die "$name: got $(stat -c %s "$dir/$name") bytes, the API says $size"
    ledger_set "$run" INSTALLER "$dir/$name"
    report "$run" "FETCHED release=$rel asset=$aid file=$name sha=$(sha256sum "$dir/$name" | cut -c1-16)"
    echo "INSTALLER $dir/$name (release $rel, asset $aid)"
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
    file="$(basename "$(ledger_get "$run" INSTALLER)")"; [ -n "$file" ] || die "vm.sh fetch or fetch-release first"
    guest="C:\\Users\\Public\\$file"
    gexec "Invoke-WebRequest -UseBasicParsing 'http://$DICTUS_HTTP_HOST:$PORT/$file' -OutFile '$guest'; (Get-Item '$guest').Length" 300
    t0="$(date +%s)"
    case "$file" in
      *.msi)
        # Per-machine: needs admin, so SYSTEM through the guest agent, not the limited token of a task
        # as dictus. Start-Process -Wait returns when msiexec has finished; 3010 is success + reboot.
        exe="$MSI_EXE"
        gexec "\$p = Start-Process msiexec.exe -ArgumentList '/i','$guest','/qn','/norestart','/l*v','C:\\Users\\Public\\dictus-msi.log' -Wait -PassThru; 'msiexec exit ' + \$p.ExitCode; if (\$p.ExitCode -in 0,3010) { exit 0 } else { exit 1 }" 300 \
          > "$run/vm-install.txt" || { cat "$run/vm-install.txt" >&2; die "msiexec failed: see $run/vm-install.txt (log in the guest: C:\Users\Public\dictus-msi.log)"; }
        ;;
      *.exe)
        # dictus.exe appears before the installer has finished writing (a launch then fails with
        # 0x80070020, sharing violation): wait for the install task itself to end.
        exe="$NSIS_EXE"
        as_user DictusVerifyInstall "$guest" '/S'
        file_s=""
        while :; do
          st="$(task_state DictusVerifyInstall)"
          if [ -z "$file_s" ] && gexec "if (Test-Path '$exe') { exit 0 } else { exit 1 }" 30 > /dev/null 2>&1; then
            file_s="$(($(date +%s) - t0))"
          fi
          case "$st" in
            "Ready 0x00000000") break ;;
            Ready\ 0x00041303 | Running* | Queued*) ;;
            *) echo "$st" > "$run/vm-install.txt"; die "install task DictusVerifyInstall ended with $st (State LastTaskResult)" ;;
          esac
          [ $(($(date +%s) - t0)) -lt 300 ] || die "install task DictusVerifyInstall still '$st' after 5 min"
          sleep 5
        done
        echo "task done $st after $(($(date +%s) - t0))s; dictus.exe present after ${file_s:-?}s" > "$run/vm-install.txt"
        ;;
      *) die "unknown installer type: $file" ;;
    esac
    gexec "if (Test-Path '$exe') { exit 0 } else { exit 1 }" 30 > /dev/null 2>&1 || die "installer finished but there is no $exe"
    ledger_set "$run" APP_EXE "$exe"
    report "$run" "INSTALLED $file in $(($(date +%s) - t0))s ($(head -1 "$run/vm-install.txt"))"
    echo "INSTALLED $exe"
    ;;
  launch)
    run="$(resolve_run)" || die "no run"
    exe="$(ledger_get "$run" APP_EXE)"; [ -n "$exe" ] || die "vm.sh install first"
    as_user DictusVerifyLaunch "$exe"
    sleep 30
    out="$(gexec "Get-Process dictus -ErrorAction SilentlyContinue | Select-Object Id,SessionId,MainWindowTitle | Format-Table -HideTableHeaders" 60 || true)"
    echo "$out" > "$run/vm-process.txt"
    pid="$(awk 'NF >= 2 && $1 ~ /^[0-9]+$/ {print $1; exit}' <<< "$out")"
    if [ -z "$pid" ]; then
      st="$(task_state DictusVerifyLaunch)"
      echo "task DictusVerifyLaunch: $st" >> "$run/vm-process.txt"
      report "$run" "LAUNCH FAILED task DictusVerifyLaunch $st"
      die "dictus.exe is not running 30 s after launch; task DictusVerifyLaunch State/LastTaskResult: ${st:-unknown}"
    fi
    report "$run" "RUNNING pid=$pid"
    echo "RUNNING pid=$pid ($run/vm-process.txt)"
    ;;
  ps) gexec "$1" "${2:-600}" ;;
  *) die "unknown command $cmd" ;;
esac

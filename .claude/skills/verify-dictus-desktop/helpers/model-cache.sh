#!/usr/bin/env bash
# A catalogue model's archive in a cache shared by every run, verified by SHA256 like the app does
# (url and sha256 read from src-tauri/src/managers/model.rs, so the cache follows the catalogue).
#   model-cache.sh <model-id>
# Last line: MODEL <id> <file> <name> (exit 0), or error: ... (exit 1). <file> is a .tar.gz for a directory
# model (Parakeet, Moonshine), extracted to models/<name>; a single file (Whisper) is copied to models/<name>.
# The cache lives outside any isolated HOME ($DICTUS_MODEL_CACHE) and is never deleted by cleanup.sh.
set -euo pipefail
source "$(dirname "$0")/env.sh"
id="${1:?usage: model-cache.sh <model-id>}"
read -r url sha dirname < <(python3 -I - "$REPO_ROOT/src-tauri/src/managers/model.rs" "$id" <<'EOF'
import re, sys
src, mid = open(sys.argv[1]).read(), sys.argv[2]
m = re.search(r'ModelInfo \{\s*id: "%s"\.to_string\(\),(.*?)\n\s*\},\n' % re.escape(mid), src, re.S)
if not m:
    sys.exit(f"error: model {mid} not in model.rs")
block = m.group(1)
url = re.search(r'url: Some\("([^"]+)"', block)
sha = re.search(r'sha256: Some\(\s*"([0-9a-f]{64})"', block)
name = re.search(r'filename: "([^"]+)"', block)
if not (url and sha and name):
    sys.exit(f"error: model {mid} has no url/sha256/filename")
print(url.group(1), sha.group(1), name.group(1))
EOF
)
[ -n "${sha:-}" ] || die "could not read $id from model.rs"
mkdir -p "$DICTUS_MODEL_CACHE"
archive="$DICTUS_MODEL_CACHE/$(basename "$url")"
if [ -f "$archive" ] && [ "$(sha256sum "$archive" | cut -c1-64)" = "$sha" ]; then
  echo "cache hit: $archive (sha256 verified)" >&2
else
  echo "downloading $url into $DICTUS_MODEL_CACHE" >&2
  curl -fL --no-progress-meter --retry 2 -o "$archive.part" "$url" >&2
  [ "$(sha256sum "$archive.part" | cut -c1-64)" = "$sha" ] || die "sha256 mismatch for $url (kept $archive.part)"
  mv "$archive.part" "$archive"
fi
echo "MODEL $id $archive $dirname"

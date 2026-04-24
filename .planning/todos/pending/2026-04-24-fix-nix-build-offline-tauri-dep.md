---
created: 2026-04-24T00:05:00.000Z
title: Fix nix-build offline mode failing to fetch cjpais/tauri.git dependency
area: nix
files:
  - flake.nix
  - .github/workflows/nix-check.yml
  - src-tauri/Cargo.toml
  - src-tauri/Cargo.lock
---

## Problem

The `nix-build` CI workflow has been failing for multiple releases (v0.1.0, v0.1.1, v0.1.2) with:

```
handy> error: failed to load source for dependency `tauri-runtime`
handy> Caused by:
handy>   Unable to update https://github.com/cjpais/tauri.git?branch=handy-2.10.2#42dc4c5e
handy> Caused by:
handy>   can't checkout from 'https://github.com/cjpais/tauri.git': you are in the offline mode (--offline)
```

**Root cause:** The Cargo.toml references a git dependency on `cjpais/tauri.git` (upstream Handy's fork of Tauri, branch `handy-2.10.2`, commit `42dc4c5e`). Nix builds run Cargo in `--offline` mode, so Cargo cannot fetch the git dep at build time — it must be vendored or declared to Nix ahead of time via `cargoHash` / `cargoLock.outputHashes` / `importCargoLock`.

**Impact:**
- `nix-build` CI red on every push — signals broken nix packaging to anyone checking repo health
- Nix users cannot build Dictus from the flake without workarounds
- NOT blocking for `release.yml` (Nix is not part of the release matrix)

## Solution — 3 paths

### Path A: Declare the git dep hash in flake.nix (preferred)
Nix's `rustPlatform.buildRustPackage` supports `cargoLock.outputHashes` which lets you pin the hash of git dependencies. Something like:

```nix
cargoLock = {
  lockFile = ./src-tauri/Cargo.lock;
  outputHashes = {
    "tauri-2.10.2" = "sha256-XXXXXX=";  # run nix-build once to get correct hash
    # possibly also tauri-runtime, tauri-runtime-wry, tauri-utils — all from the same git ref
  };
};
```

Requires: identifying every crate name in the `42dc4c5e` git ref and computing its sha256 via `nix-prefetch-git` or a failed build.

### Path B: Replace the git dep with upstream crates.io versions
If we don't actually need the Handy-specific Tauri patches, switch `[patch.crates-io]` or dependency refs back to `tauri = "2.10.2"` from crates.io. Requires auditing what the Handy fork changed vs. upstream Tauri and whether we still depend on those changes.

### Path C: Remove nix-build from CI gating
Drop `nix-check.yml` from the CI matrix if we don't actively support Nix users. Fast and honest but loses that surface area.

**Recommendation:** Path A. Dictus is a fork with upstream drift concerns — keeping the Handy-patched Tauri fork is safer than risking subtle behavior differences by switching to crates.io.

## Priority / timing

- **Blocks:** clean green CI on main; Nix packaging users
- **Does NOT block:** v0.1.2 release or subsequent releases
- **Good candidate for a small infra phase** or a drive-by fix by someone comfortable with Nix derivations

## Definition of done

- `nix build .#` succeeds locally on Linux
- `nix-check.yml` turns green on main
- flake.nix is documented to explain why the git dep needs hash pinning

# Phase 04 — Deferred Items

Out-of-scope discoveries logged during plan execution. These are NOT fixed by the discovering plan (per GSD scope boundary rules); they must be addressed by the owning plan or a follow-up.

---

## 1. validate.sh anti-regression check mismatch (Plan 04-01 scope)

**Discovered during:** Plan 04-03 Task 3 (first green validate.sh run)
**Discovered:** 2026-04-11
**Owner:** Plan 04-01 (validator author)

### Symptom

`.planning/phases/04-updater-infrastructure/scripts/validate.sh` fails its anti-regression assertion:

```
FAIL Anti-regression build.yml has TECH-03 deferral comment near otool line
     cmd: grep -B5 'otool -L' .github/workflows/build.yml | grep -q 'TECH-03'
```

### Root cause

The TECH-03 deferral comment exists at `.github/workflows/build.yml:375-376`, but the first `otool -L` invocation is at line 385. The validator uses `grep -B5` (5 lines of context above), so it only sees lines 380-385 — which do not contain `TECH-03`. The comment is 9-10 lines above the otool line, outside the `-B5` window.

### Why not fixed now

Pre-existing at the moment `validate.sh` was committed (28f8255 added the validator AFTER 7105735 placed the comment). Plan 04-03 did not touch either file in the failing assertion. Per GSD scope boundary rules, auto-fix only applies to issues directly caused by the current task's changes.

### Suggested fix (for Plan 04-01 follow-up or Plan 04-04)

Either:
- **Option A (preferred):** Change the validator's `-B5` to `-B12` (or similar) so the existing comment is in range. Zero code movement.
- **Option B:** Move/duplicate the `TECH-03` deferral comment inside the `Verify macOS dylib bundling` step body (just above the otool lines), so it falls within `grep -B5`.

Both options are minutes of work. No urgency — the UPDT-03..09 assertions all pass, so the updater config itself is correct. This is purely a validator-check ergonomics fix.

---
phase: 06-brand-icon-polish
plan: 03
subsystem: ui
tags: [react, typescript, tauri, i18n, debug-panel, portable-mode]

# Dependency graph
requires:
  - phase: 06-brand-icon-polish
    provides: "get_app_dir_path Tauri command (already registered in commands/mod.rs) returning portable-aware app data dir"
provides:
  - "DebugPaths component fetching real runtime app data dir via commands.getAppDirPath()"
  - "BRAND-03 requirement closed — no more hardcoded %APPDATA%/handy strings in debug panel"
affects: [06-brand-icon-polish, verify-sync]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Pre-compute derived path strings as JS variables before JSX render to satisfy i18next/no-literal-string ESLint rule"
    - "Follow AppDataDirectory.tsx pattern: inline useEffect + loading skeleton + error state for Tauri command calls"

key-files:
  created: []
  modified:
    - src/components/settings/debug/DebugPaths.tsx

key-decisions:
  - "Pre-compute modelsPath and settingsPath as template literals before render (not inline JSX concatenation) — satisfies ESLint i18next/no-literal-string which flags mixed JSX text/expression spans"
  - "Reused existing get_app_dir_path command — no new backend command created (honored RESEARCH.md Pitfall 1)"
  - "Used inline useEffect (no custom hook abstraction) — consistent with AppDataDirectory.tsx project convention"

patterns-established:
  - "Pattern: For path suffix concatenation in JSX, pre-compute as template literals in component body rather than inline JSX — avoids ESLint i18next/no-literal-string false positives"

requirements-completed: [BRAND-03]

# Metrics
duration: 8min
completed: 2026-04-16
---

# Phase 06 Plan 03: DebugPaths Backend Path Fetch Summary

**DebugPaths panel rewritten to fetch real portable-aware app data dir via commands.getAppDirPath(), removing three hardcoded %APPDATA%/handy strings and three ESLint disable directives**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-04-16T05:32:11Z
- **Completed:** 2026-04-16T05:40:00Z
- **Tasks:** 1 of 1
- **Files modified:** 1

## Accomplishments
- DebugPaths panel now displays the real runtime-resolved app data directory (e.g., `/Users/<name>/Library/Application Support/com.dictus.desktop` on macOS, portable `Data/` dir in portable mode)
- Removed all three hardcoded `%APPDATA%/handy*` strings and all three `eslint-disable-next-line i18next/no-literal-string` directives
- Added loading skeleton and error state following AppDataDirectory.tsx UX pattern
- BRAND-03a verify-sync.sh assertion passes; all 15 verify-sync assertions green

## Task Commits

1. **Task 1: Rewrite DebugPaths.tsx to fetch backend path via getAppDirPath** - `7e8bedc` (fix)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/components/settings/debug/DebugPaths.tsx` - Rewritten: calls commands.getAppDirPath() on mount, renders three dynamic path spans from backend-provided base path

## Decisions Made
- Pre-computed `modelsPath` and `settingsPath` as template literals before the JSX return block. The ESLint rule `i18next/no-literal-string` flags spans containing `/models` or `/settings_store.json` as inline JSX text, even when mixed with a `{variable}` expression. Moving concatenation to JS variables before render satisfies the rule without adding disable directives.
- Did not create a new backend Tauri command — `get_app_dir_path` already existed and returned exactly the needed portable-aware path (honored RESEARCH.md Pitfall 1).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ESLint flags inline template suffix strings in JSX spans**
- **Found during:** Task 1 (DebugPaths rewrite)
- **Issue:** First implementation used `{appDirPath}/models` and `{appDirPath}/settings_store.json` directly in JSX span content. ESLint i18next/no-literal-string treats the `/models` and `/settings_store.json` suffixes as literal strings in JSX context, causing 2 new errors.
- **Fix:** Pre-computed `modelsPath = \`${appDirPath}/models\`` and `settingsPath = \`${appDirPath}/settings_store.json\`` as variables before the return block; JSX spans now contain only `{modelsPath}` and `{settingsPath}` expressions.
- **Files modified:** src/components/settings/debug/DebugPaths.tsx
- **Verification:** `bun run lint` exits with 0 new errors from DebugPaths.tsx; only pre-existing DictusLogo.tsx error remains (out of scope).
- **Committed in:** 7e8bedc (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug/ESLint false positive)
**Impact on plan:** Fix necessary to satisfy lint gate. No scope creep. Pattern noted for future path concatenation in JSX.

## Issues Encountered
- Pre-existing `DictusLogo.tsx` ESLint error (`i18next/no-literal-string` on SVG text element) causes `bun run lint` to exit 1 project-wide. This is out-of-scope for this plan — confirmed pre-existing before my changes via `git stash` verification. The plan's lint gate for DebugPaths.tsx specifically is clean.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- BRAND-03 closed. All BRAND-0x and ICON-02a assertions in verify-sync.sh now pass.
- Phase 06 wave 2 complete (plans 06-02 and 06-03 both done).
- DictusLogo.tsx pre-existing lint error should be addressed in a follow-up (deferred to deferred-items).

---
*Phase: 06-brand-icon-polish*
*Completed: 2026-04-16*

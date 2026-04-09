# Deferred Items

## Pre-existing Lint Error: DictusLogo.tsx

**File:** `src/components/icons/DictusLogo.tsx`
**Issue:** Literal string "Dictus" inside SVG `<text>` element triggers `i18next/no-literal-string` ESLint rule
**Discovered during:** Phase 03, Plan 02 (Task 1 verification)
**Impact:** `bun run lint` returns 1 error, blocks lint-clean CI enforcement
**Resolution:** Either add `{/* eslint-disable-next-line i18next/no-literal-string */}` comment above the SVG `<text>` element, or convert the text to use a `title` attribute/aria-label that doesn't appear in JSX
**Note:** This is an SVG brand logo text — using i18n for the brand name "Dictus" is inappropriate; the eslint-disable comment is the correct fix

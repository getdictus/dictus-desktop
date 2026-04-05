# Testing Patterns

**Analysis Date:** 2026-04-05

## Test Framework

**Runner:**
- Playwright 1.58.0 (for E2E browser testing)
- Config: `playwright.config.ts`

**Assertion Library:**
- @playwright/test (includes assertions via `expect()`)

**Run Commands:**
```bash
bun run test:playwright         # Run all Playwright tests
bun run test:playwright:ui      # Run tests in interactive UI mode
```

**Note:** No unit test framework (Jest/Vitest) detected. Testing is currently E2E only.

## Test File Organization

**Location:**
- E2E tests: `tests/` directory at project root
- Naming: `*.spec.ts` pattern (e.g., `tests/app.spec.ts`)
- All tests located in single `tests/` directory (flat structure)

**Structure:**
```
tests/
└── app.spec.ts              # E2E browser tests
```

**Current Coverage:**
- Single test file with 2 basic sanity tests
- No unit tests for backend (Rust) or frontend (React components)
- No integration tests between frontend and backend

## Test Structure

**Suite Organization (Playwright):**
```typescript
import { test, expect } from "@playwright/test";

test.describe("Handy App", () => {
  test("test name", async ({ page }) => {
    // Test implementation
  });
});
```

**Patterns:**
- `test.describe()` groups related tests
- `test()` or `test.only()` for individual tests
- Async function with `page` fixture for browser interaction
- `page.goto()` for navigation
- `expect()` for assertions

## Playwright Configuration

**baseURL:** `http://localhost:1420` (dev server)

**Web Server:**
- Command: `bunx vite dev`
- URL: `http://localhost:1420`
- Reuse: Existing server reused unless in CI
- Timeout: 30 seconds for startup

**Retry Logic:**
- Development: No retries (first failure stops test)
- CI: 2 retries for flaky test handling

**Parallelization:**
- Development: Uses all available CPU cores (`workers: undefined`)
- CI: Single worker only (prevents resource contention)

**Reporter:**
- Format: HTML report (`reporter: "html"`)
- Trace: Recorded on first failure (`trace: "on-first-retry"`)

**Browser Coverage:**
- Chromium only (single project)
- Full device emulation not configured

## Existing Tests

**File:** `tests/app.spec.ts`

**Test 1: "dev server responds"**
```typescript
test("dev server responds", async ({ page }) => {
  const response = await page.goto("/");
  expect(response?.status()).toBe(200);
});
```
- Purpose: Verify dev server is running and responding to requests
- Validates: HTTP 200 status on root path
- Scope: Infrastructure check only

**Test 2: "page has html structure"**
```typescript
test("page has html structure", async ({ page }) => {
  await page.goto("/");
  const html = await page.content();
  expect(html).toContain("<html");
  expect(html).toContain("<body");
});
```
- Purpose: Verify basic HTML structure renders
- Validates: Presence of `<html>` and `<body>` tags
- Scope: Very basic sanity check

## Mocking

**Framework:** Playwright does not require explicit mocking for:
- Network requests (can intercept via `page.route()`)
- Timers (built-in `page.clock`)
- Tauri IPC (uses real backend during tests)

**Current Approach:**
- No mocking implemented in existing tests
- Tests run against real dev server and Tauri backend
- No stubbing of API responses

**What to Mock (if adding unit tests):**
- Zustand store actions (use `create()` with test configuration)
- Tauri commands (import and stub `commands` object)
- API responses (use Playwright's `page.route()`)
- Event listeners (mock `listen()` from `@tauri-apps/api/event`)

**What NOT to Mock:**
- Core React hooks (useState, useEffect, useContext)
- Browser APIs (localStorage, sessionStorage unless testing persistence)
- DOM rendering (test against real DOM, not virtual snapshots)

## Fixtures and Factories

**Test Data:**
- No fixtures or test factories currently defined
- Tests use live data from dev server/backend

**Location:**
- Would be placed in `tests/fixtures/` or `tests/factories/` if created
- Not yet implemented

**Recommendation for Future Tests:**
```typescript
// Example factory pattern for creating test data
const createMockAudioDevice = (overrides = {}) => ({
  index: "default",
  name: "Default",
  is_default: true,
  ...overrides,
});

const createMockSettings = (overrides = {}) => ({
  always_on_microphone: false,
  selected_microphone: "default",
  // ... other settings
  ...overrides,
});
```

## Coverage

**Requirements:** None enforced

**View Coverage:** Not configured

**Current State:**
- No coverage measurement in place
- No minimum coverage threshold
- No coverage reports generated

**Gaps:**
- Backend (Rust) has 0% test coverage
- Frontend unit tests: 0% coverage
- Frontend E2E: Very minimal (2 basic sanity tests)
- Complex features untested: Settings management, audio recording, model downloads, shortcuts

## Test Types

**Unit Tests:**
- Scope: Not implemented
- Would test: Individual functions, components in isolation
- Framework: Would use Vitest or Jest if implemented
- Approach: Mock Tauri commands and Zustand store

**Integration Tests:**
- Scope: Not implemented
- Would test: Frontend ↔ Backend communication via Tauri IPC
- Would validate: Settings persistence, model management flow
- Approach: Use test Tauri instance or mock IPC layer

**E2E Tests:**
- Framework: Playwright (only test type currently used)
- Scope: User workflows in real browser against dev server
- Examples needed:
  - Onboarding flow (accessibility → model selection → main app)
  - Settings changes and persistence
  - Audio recording and transcription
  - Language switching
  - App window interactions

## Common Patterns (Future Reference)

**Async Testing (Playwright):**
```typescript
test("async operation", async ({ page }) => {
  // Wait for navigation to complete
  await page.goto("/");
  
  // Wait for element to be visible
  await page.locator("button").waitFor({ state: "visible" });
  
  // Click and wait for navigation
  await Promise.all([
    page.waitForNavigation(),
    page.click("a#next-page"),
  ]);
  
  // Wait for specific condition
  await expect(page.locator(".loaded")).toBeVisible();
});
```

**Error Testing:**
```typescript
test("error handling", async ({ page }) => {
  // Intercept and block request to simulate error
  await page.route("**/api/settings", (route) => {
    route.abort("failed");
  });
  
  await page.goto("/");
  
  // Verify error message displayed
  await expect(page.locator(".error-toast")).toBeVisible();
  await expect(page.locator(".error-toast")).toContainText("Failed");
});
```

**Component Interaction Testing:**
```typescript
test("dropdown selection", async ({ page }) => {
  await page.goto("/");
  
  // Open dropdown
  await page.click("button:has-text('Select microphone')");
  
  // Select option
  await page.click("text=Built-in Microphone");
  
  // Verify selection applied
  await expect(page.locator("[value='Built-in Microphone']")).toBeVisible();
});
```

## Frontend Testing Gaps

**UI Component Testing Needed:**
- `Button`, `Dropdown`, `Input`, `Slider`, `ToggleSwitch` - basic prop combinations
- `SettingContainer` - with/without description modes
- Settings pages - option selection, value updates, reset functionality
- Model selector - download progress, status transitions

**Hook Testing Needed:**
- `useSettings` - initialization, setting updates, error handling
- `useOsType` - platform detection
- Translation hooks - language switching

**Store Testing Needed (Zustand):**
- `settingsStore.ts` - all 40+ action functions
- State mutations and side effects
- Error recovery and rollback behavior

**User Flow Testing Needed:**
- Complete onboarding sequence (permissions → model → settings)
- Recording and transcription workflow
- Settings persistence across app restarts
- Language and theme switching
- Audio device hot-plugging

## CI/CD Integration

**CI Behavior:**
- Runs in single-worker mode to avoid resource contention
- Retries failed tests 2 times for flaky test resilience
- Forbids `test.only()` and `test.skip()` in CI
- Generates HTML reports and traces on failure

**Local Development:**
- Full parallelization (all cores)
- No retries (fail fast)
- Reuses existing dev server if running

---

*Testing analysis: 2026-04-05*

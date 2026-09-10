<!--
Thanks for contributing to Dictus Desktop.

Small, focused PRs get reviewed fastest. If you are planning something large,
open an issue or a discussion first so we can agree on the approach before you
write the code.
-->

## What does this change?

<!--
Describe the problem you hit or the idea you had, and what you did about it.
Two or three sentences in your own words is plenty.

AI-assisted PRs are welcome — this project is built with AI assistance too. But
we want to read your reasoning, not a generated summary. If you cannot explain
why the change is right, it is not ready for review.
-->

## Related issues

<!-- Use "Fixes #123" to close an issue automatically, or "Related: #123" otherwise. -->

Fixes #

## How did you test it?

<!-- What you actually ran, and what you saw. "It builds" is not testing. -->

**Platforms tested:**

- [ ] macOS (Apple Silicon)
- [ ] macOS (Intel)
- [ ] Windows
- [ ] Linux

Dictus behaves differently per platform — audio capture, global shortcuts, and
paste all use OS-specific paths. Tell us which platforms you could not test so a
maintainer can cover them.

## Checklist

- [ ] I searched [existing issues](https://github.com/getdictus/dictus-desktop/issues) and [pull requests](https://github.com/getdictus/dictus-desktop/pulls), including closed ones
- [ ] I read [CONTRIBUTING.md](https://github.com/getdictus/dictus-desktop/blob/main/CONTRIBUTING.md)
- [ ] `bun run lint` and `bun run format:check` pass
- [ ] `cargo fmt --check` and `cargo clippy` pass for Rust changes
- [ ] New user-facing strings go through i18next (`src/i18n/locales/en/translation.json`) — no hardcoded text in JSX

## Screenshots or video

<!-- Required for any visible UI change. A short screen recording beats a paragraph. -->

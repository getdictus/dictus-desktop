# Contributing to Dictus Desktop

Thank you for your interest in contributing to Dictus Desktop! This guide will help you get started with contributing to this open source speech-to-text application.

## 📖 Philosophy

Dictus Desktop aims to be a useful, privacy-first speech-to-text app and a solid foundation for contributors to build on. We prioritize:

- **Simplicity**: Clear, maintainable code over clever solutions
- **Extensibility**: Make it easy for others to understand and customize
- **Privacy**: Keep speech processing local and offline
- **Accessibility**: Make useful speech-to-text tooling available to more people

## 🚀 Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) package manager
- Platform-specific build tools (see [BUILD.md](BUILD.md))

### Setting Up Your Development Environment

1. **Fork the repository** on GitHub.

2. **Clone your fork**:

   ```bash
   git clone git@github.com:YOUR_USERNAME/dictus-desktop.git
   cd dictus-desktop
   ```

3. **Add the upstream remote**:

   ```bash
   git remote add upstream git@github.com:getdictus/dictus-desktop.git
   ```

4. **Install dependencies**:

   ```bash
   bun install
   ```

5. **Download required models**:

   ```bash
   mkdir -p src-tauri/resources/models
   curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx
   ```

6. **Run in development mode**:

   ```bash
   bun run tauri dev
   # On macOS if you encounter cmake errors:
   CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev
   ```

For detailed platform-specific setup instructions, see [BUILD.md](BUILD.md).

### Understanding the Codebase

**Backend (Rust - `src-tauri/src/`):**

- `lib.rs` - Main application entry point with Tauri setup
- `managers/` - Core business logic (audio, model, transcription)
- `audio_toolkit/` - Low-level audio processing (recording, VAD)
- `commands/` - Tauri command handlers for frontend communication
- `shortcut.rs` - Global keyboard shortcut handling
- `settings.rs` - Application settings management

**Frontend (React/TypeScript - `src/`):**

- `App.tsx` - Main application component
- `components/` - React UI components
- `hooks/` - Reusable React hooks
- `lib/types.ts` - Shared TypeScript types

For more details, see the Architecture section in [README.md](README.md) or [AGENTS.md](AGENTS.md).

## 🐛 Reporting Bugs

Before submitting a bug report:

1. Search [existing issues](https://github.com/getdictus/dictus-desktop/issues).
2. Check [Discussions](https://github.com/getdictus/dictus-desktop/discussions).
3. Try the latest release to see whether the issue has already been fixed.
4. Enable debug mode (`Cmd/Ctrl+Shift+D`) when useful for gathering diagnostic information.

When reporting a bug, include your app version, operating system, relevant hardware, reproduction steps, expected behavior, actual behavior, and screenshots or logs when available.

Use the [Bug Report template](.github/ISSUE_TEMPLATE/bug_report.md) when creating an issue.

## 💡 Suggesting Features

Use [GitHub Discussions](https://github.com/getdictus/dictus-desktop/discussions) for feature ideas and larger changes before opening a PR. Describe the problem you want to solve, your proposed solution, alternatives you considered, and why the change would benefit Dictus Desktop users.

## 🔧 Making Code Contributions

### Before You Start

Search existing issues and pull requests first:

- [Open issues](https://github.com/getdictus/dictus-desktop/issues)
- [Closed issues](https://github.com/getdictus/dictus-desktop/issues?q=is%3Aissue+is%3Aclosed)
- [Open pull requests](https://github.com/getdictus/dictus-desktop/pulls)
- [Closed pull requests](https://github.com/getdictus/dictus-desktop/pulls?q=is%3Apr+is%3Aclosed)

For larger features or changes that have not already been requested, start a discussion first so maintainers and community members can give feedback before significant implementation work begins.

### Development Workflow

1. **Create a branch**:

   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. **Make focused changes** that follow the existing code style and patterns.

3. **Test your changes** on the platforms and scenarios you can reasonably cover.

4. **Commit your changes** using a clear conventional commit message:

   ```bash
   git add .
   git commit -m "feat: add your feature description"
   # or
   git commit -m "fix: describe the bug fix"
   ```

   Common prefixes include `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, and `chore:`.

5. **Keep your fork updated**:

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

6. **Push your branch**:

   ```bash
   git push origin your-branch-name
   ```

7. **Create a Pull Request** against [getdictus/dictus-desktop](https://github.com/getdictus/dictus-desktop) and explain what changed, why it changed, how you tested it, and any limitations or untested platforms.

### AI Assistance

AI-assisted contributions are welcome. If you used an AI tool, disclose it in the PR description and briefly explain how it was used so reviewers have appropriate context.

### Code Style Guidelines

**Rust:**

- Follow standard Rust formatting (`cargo fmt`)
- Run `cargo clippy` and address warnings
- Use descriptive variable and function names
- Add doc comments for public APIs
- Handle errors explicitly and avoid unnecessary `unwrap` calls in production code

**TypeScript/React:**

- Use TypeScript strictly and avoid `any` where possible
- Follow React hooks best practices
- Use functional components
- Keep components small and focused
- Use Tailwind CSS for styling
- Route every user-facing string through i18next — add the key to `src/i18n/locales/en/translation.json` and read it with `t('key.path')`. ESLint fails on hardcoded strings in JSX

**General:**

- Write self-documenting code
- Add comments for non-obvious logic
- Keep functions focused
- Prioritize readability over cleverness

## 🧪 Testing Your Changes

Run the repository's lint and formatting tools before submitting code changes:

```bash
bun run lint
bun run format:check
```

`bun run format` rewrites files in place; `format:check` only reports, which is what CI runs.

Run the app in development mode when appropriate:

```bash
bun run tauri dev
```

For production-build verification:

```bash
bun run tauri build
```

For documentation-only changes, verify links, commands, formatting, and rendered Markdown rather than running the full application test suite unless the change affects executable code.

## 📝 Documentation Contributions

Documentation improvements are welcome, including updates to README.md, BUILD.md, CONTRIBUTING.md, tutorials, code comments, and user-facing guidance.

## 🤝 Community Guidelines

- Be respectful and inclusive
- Be patient with maintainers and other contributors
- Be constructive and solution-focused
- Search existing issues, PRs, and discussions before creating duplicates

## 🎯 Good First Issues

If you're new to the project, look for issues labeled `good first issue` or `help wanted`. These are usually smaller, well-scoped tasks that are good entry points into the codebase.

## 📞 Getting Help

- [Telegram community](https://t.me/getdictus) — quickest way to reach maintainers and other users
- [GitHub Discussions](https://github.com/getdictus/dictus-desktop/discussions)
- [Repository issues](https://github.com/getdictus/dictus-desktop/issues)

## 📜 License

By contributing to Dictus Desktop, you agree that your contributions will be licensed under the repository's license. See [LICENSE](LICENSE) for details.

---

**Thank you for contributing to Dictus Desktop!**

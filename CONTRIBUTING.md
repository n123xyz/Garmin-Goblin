# Contributing to Garmin Goblin 👹🤝

Thank you for your interest in contributing to **Garmin Goblin**! Whether you are optimizing on-device neural runtimes, polishing the Svelte 5 frontend, expanding reverse-engineered Garmin BLE protocols, or improving FIT file parsing, we welcome your contributions.

---

## Code of Conduct

All contributors are expected to uphold our [Code of Conduct](CODE_OF_CONDUCT.md). Please treat fellow community members with kindness and respect.

---

## How Can You Contribute?

1. **Reporting Bugs**: Check the [Issues tracker](https://github.com/n123xyz/Garmin-Goblin/issues) to ensure the bug has not already been reported. When filing, include device model, Android OS version, and relevant logs (`adb logcat`).
2. **Feature Suggestions**: Open a feature request discussing your concept before beginning large architectural refactors.
3. **Documentation**: Improve guides in `docs/` or clarify setup instructions.
4. **Code**: Submit Pull Requests implementing bug fixes or approved features.

---

## Development Workflow

1. **Fork the repository** on GitHub.
2. **Clone your fork**:
   ```bash
   git clone https://github.com/<your-username>/Garmin-Goblin.git
   cd Garmin-Goblin
   ```
3. **Create a descriptive feature branch**:
   ```bash
   git checkout -b feat/my-new-feature
   ```
4. **Install dependencies**:
   ```bash
   pnpm install
   ```
5. **Run validation checks before committing**:
   ```bash
   # Svelte / TypeScript typechecking
   pnpm run check

   # Rust linting & formatting
   cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
   ```
6. **Commit and push**:
   Use clear, conventional commit messages (`feat: ...`, `fix: ...`, `docs: ...`).
7. **Submit a Pull Request** against the `main` branch.

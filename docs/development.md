# Developer Guide 💻

This guide provides technical guidelines for developers looking to contribute, modify schemas, or extend **Garmin Goblin**.

---

## 1. Codebase Architecture Overview

```text
garmin-goblin/
├── docs/                        # Technical documentation & architecture guides
├── website/                     # Svelte 5 + Vite official project website
├── src/                         # Svelte 5 (Runes) frontend
│   ├── lib/
│   │   ├── components/          # Reusable UI widgets (GarminDeviceCard, LeafletMap, etc.)
│   │   ├── data/                # Static data & cognitive test condition parameters
│   │   ├── services/            # Client engines (caritEngine, facenameEngine, vismotorEngine)
│   │   ├── state/               # Reactive Svelte 5 runes ($state)
│   │   └── types/               # TypeScript interfaces
│   └── routes/                  # SvelteKit application routes (biometrics, therapy, cognitive tests)
├── src-tauri/                   # Rust Tauri v2 native core
│   ├── src/
│   │   ├── ai/                  # LiteRT NPU adapter & Ollama desktop adapter
│   │   ├── db/                  # SQLite schema creation & linear PRAGMA migrations
│   │   ├── services/            # Rust business logic (garmin_ble, garmin_mtp, fit_parser, goblin)
│   │   ├── lib.rs               # Tauri entry point & command registration
│   │   └── main.rs              # Desktop executable harness
│   └── gen/android/             # Android Gradle project & native JNI glue
└── plugins/tauri-plugin-litert/ # Custom Tauri plugin managing Kotlin LiteRT on Android
```

---

## 2. Database Migrations

Persistence is managed by `rusqlite`. Never alter existing migration steps in `src-tauri/src/db/mod.rs`! Always append new migrations by incrementing `user_version`:

```rust
// Example: Adding a new migration in src-tauri/src/db/mod.rs
let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

if current_version < 7 {
    conn.execute_batch(
        "BEGIN;
         CREATE TABLE IF NOT EXISTS new_metric (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             timestamp TEXT NOT NULL,
             val REAL NOT NULL
         );
         PRAGMA user_version = 7;
         COMMIT;"
    )?;
}
```

---

## 3. Registering New Tauri IPC Commands

1. Define your command function in an appropriate module under `src-tauri/src/services/`.
2. Ensure arguments and return types implement `serde::Serialize` and `serde::Deserialize`.
3. Register the function with `tauri::generate_handler![]` in `src-tauri/src/lib.rs`.
4. Define matching TypeScript types in `src/lib/types/` and invoke via `@tauri-apps/api/core`:
   ```typescript
   import { invoke } from '@tauri-apps/api/core';
   const result = await invoke<MyType>('my_new_command', { param: value });
   ```

---

## 4. Code Quality & Testing Protocols

Before submitting pull requests or building release binaries, always run the validation suite:

### TypeScript & Svelte Verification:
```bash
pnpm run check
```
Must produce `0 errors and 0 warnings`.

### Rust Clippy & Linter:
```bash
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```
Must pass with zero warnings or errors.

### Android Code Formatting:
Follow standard Kotlin coding conventions for any changes made in `plugins/tauri-plugin-litert/android` or `src-tauri/gen/android`.

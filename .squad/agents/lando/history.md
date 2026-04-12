# Lando — History

## Project Context

**Project:** work-tracker-2 — Native desktop time tracker for consultants
**User:** Fredrik Kristiansen Wikestad
**Stack:** Tauri 2 + Rust + SQLite + Svelte 5 + TypeScript
**Joined:** 2026-04-12

The app is a Tauri 2 desktop app. Building requires:
- Rust toolchain (stable) + cargo
- Node.js + npm
- Platform-specific build tools (MSVC on Windows, Xcode on macOS, etc.)

Build commands:
- `npm run build` — frontend (vite)
- `cd src-tauri && cargo build` — Rust backend
- `npm run tauri:build` — full Tauri bundle

Test commands:
- `cd src-tauri && cargo test` — 16 Rust integration tests (all passing as of 2026-04-12)
- `npm test` — Vitest frontend tests (2 tests; some $effect tests deferred to Phase 2)

No CI pipelines exist yet — `.github/workflows/` may not exist.

Key config files:
- `src-tauri/Cargo.toml` — Rust deps and features
- `package.json` — frontend scripts and deps
- `vitest.config.ts` — Vitest configuration
- `src-tauri/tauri.conf.json` — Tauri app config

## Learnings

_Populated as Lando works on the project._

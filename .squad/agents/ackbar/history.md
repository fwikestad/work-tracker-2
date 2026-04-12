# Ackbar — History

## Project Context

**Project:** work-tracker-2 — Native desktop time tracker for consultants
**User:** Fredrik Kristiansen Wikestad
**Stack:** Tauri 2 + Rust + SQLite (rusqlite) + Svelte 5 + TypeScript
**Joined:** 2026-04-12

The app is a local-only desktop app. No cloud services, no network exposure. Data lives in a SQLite DB on the user's machine. Tauri 2 provides the native wrapper — Rust backend handles all DB operations via IPC commands, Svelte 5 frontend renders the UI.

Key architecture facts:
- IPC commands in `src-tauri/src/commands/` — these are the attack surface boundary
- DB access via `get_conn()` helper (introduced 2026-04-12 refactor) — safe Mutex lock acquisition
- `tauri-plugin-fs` and `tauri-plugin-dialog` are in use — file system access present
- WAL mode enabled, foreign keys enforced
- No authentication layer (single-user local app)
- 16 Rust integration tests passing, Vitest frontend tests present

## Learnings

_Populated as Ackbar works on the project._

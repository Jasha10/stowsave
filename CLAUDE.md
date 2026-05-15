# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

- Build: `cargo build`
- Run all tests: `cargo test` (or `just test`)
- Run a single test: `cargo test <test_name>` (e.g. `cargo test test_create_backup`)
- Format: `just fmt` (uses `cargo +nightly fmt` — nightly toolchain required)
- Regenerate `README.md`: `just readme` (extracts `//!` doc comments from `src/main.rs`)

## Architecture

`stowsave` is a small Rust CLI that moves a file/dir into a GNU Stow package and lets `stow` re-create a symlink at the original location, with a `.bak` copy of the original.

The central design is a **plan/execute split** in `src/main.rs`:

1. `collect_commands(&args)` canonicalizes paths, runs all `checks::*` validators, and builds a `Vec<Command>`. Pure planning — no filesystem mutation.
2. `execute_commands(commands, verbose)` invokes each `Command` in order.

This means validation failures surface before any side effects, so a partial-failure mid-run is much less likely.

Module map:

- `src/main.rs` — `clap` CLI entry, `collect_commands`, `execute_commands`. The crate-level `//!` doc comments at the top are the source of truth for `README.md`.
- `src/command.rs` — `Command` enum (`CreateDirIfNotExists`, `MoveToDir`, `CreateBackup`, `RunStow`) and the `CommandImpl::invoke` impl. All filesystem side effects live here. `RunStow` shells out to the external `stow` binary.
- `src/checks.rs` — pre-flight validators returning `Result<()>`. `StowSaveError` (thiserror) holds the typed error variants. **Key invariant:** `stow_directory_is_grandchild_of_common_ancestor` enforces that the stow package is exactly two levels below the common ancestor of `path_to_save` and `stow_package`. This is what makes the relative path inside the stow package mirror the original location so `stow` re-creates the symlink at the right place.
- `src/util.rs` — `find_common_ancestor` over absolute paths only (asserts absoluteness).

## Gotchas

- `README.md` is **generated** from the `//!` comments at the top of `src/main.rs`. Edit the doc comments, then run `just readme`. Direct edits to `README.md` will be overwritten.
- `tests/e2e_tests.rs` is currently entirely commented out. The roadmap (top of `src/main.rs`) calls for real e2e tests — treat this file as a known gap, not dead code to delete.
- External dependency at runtime: the `stow` binary must be on `PATH` for `RunStow` to succeed. The `test_run_stow` unit test in `src/command.rs` skips itself when `stow` is unavailable.
- CI (`.github/workflows/rust_build_test.yaml`) runs `cargo build` + `cargo test` on `ubuntu-latest` and `macos-latest` for pushes and PRs to `main`.

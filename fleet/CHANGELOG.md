# Changelog

Release convention: every push to `main` requires a matching versioned note under
`docs/releases/` and an entry in this file. See [CONTRIBUTING.md](CONTRIBUTING.md).

## [0.1.2] - 2026-09-28

### Added

- `fleet swarm --prompt-file <PATH>` reads the worker instructions from a `.txt`, `.md`, or
  `.pdf` file instead of an inline `--prompt`. PDF text is extracted in-process
  (`pdf-extract`), with no external tool required.

### Changed

- An unsupported, missing, empty, non-UTF-8, textless, malformed, or oversized prompt file is
  refused with exit 7 naming the path. It is never silently replaced by `--task`'s text.
  `--prompt-file` and `--prompt` together are a usage error.

### Verification

- Local only (macOS arm64, rustc 1.98.1); no CI.
- New tests: 6 unit + 3 real-binary, all pass; the 8 existing swarm unit tests still pass.
- `cargo test --workspace`: 846 passed, 17 failed; the same 17 fail on untouched `main`.
- `cargo clippy -D warnings` fails on untouched `crates/plan` with clippy 1.98; 0 warnings in
  changed files.
- Real `fleet swarm --agent freelane --prompt-file <pdf> --merge` built the file the PDF
  described (exit 0).

### Known limitation

- Swarm's argument-validation refusals, including this flag's, still exit 7 without a ledger
  receipt (AGENTS.md rule 8), as the existing `--role`/`--task` refusals already do.
- Scanned PDFs are refused, not OCR'd; multi-column PDFs may extract out of reading order.

## [0.1.1] - 2026-09-15

### Added

- Human-readable LLD execution tracing for `fleet run`.
- Numbered LLD path logs for every stage and node, including crash-resumed stages.
- Interactive-answer tracing that identifies the direct adapter path and states when the full
  pipeline is not traversed.
- Native Apple Silicon development artifacts under `target/aarch64-apple-darwin`.

### Fixed

- `cargo-watch` now invokes `scripts/fleet-build.sh` through shell mode instead of incorrectly
  dispatching `cargo bash`.
- Adopted legacy watchers now use their existing `.dev-watch/watch.log` for diagnostics.
- `./dev.sh watch` stays in watcher mode instead of forwarding `watch` to Fleet.
- ARM64 development no longer selects the x86_64 Rust toolchain when the native toolchain is
  installed.
- The optional `sccache` wrapper is opt-in via `FLEET_USE_SCCACHE=1`; the default avoids a
  stalled ARM64 Cargo/x86_64 toolchain combination.

### Verification

- Native binary confirmed as `Mach-O 64-bit executable arm64`.
- `cargo test -p print`: 19 passed.
- Repeat native build: 0.436 seconds after the initial 63-second build.
- Real `fleet run` printed the LLD path trace through Verify.

### Known limitation

- The real pipeline run reached Verify but the repository secret scanner timed out, so that run
  is not recorded as a full pipeline pass.

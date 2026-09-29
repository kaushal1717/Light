# Changelog

Release convention: every push to `main` requires a matching versioned note under
`docs/releases/` and an entry in this file. See [CONTRIBUTING.md](CONTRIBUTING.md).

## [0.1.2] - 2026-09-28

### Added

- `fleet swarm --prompt-file <PATH>` reads the worker instructions from a `.txt`, `.md`, or
  `.pdf` file instead of an inline `--prompt`. PDF text is extracted by `pdf-extract` in an
  isolated `fleet __pdf-text` child with a 256 MiB heap cap and a 30 s deadline, so a PDF that
  inflates to gigabytes is refused (exit 7) instead of exhausting the CLI's memory. No external
  tool is required.

### Fixed

- A prompt starting with `-` (e.g. a plan's `---` front matter) no longer kills the `__agent`
  child with a clap usage error. Before, the lane ended "agent exited without an fd-3 result".

### Changed

- An unsupported, missing, empty, non-UTF-8, textless, malformed, or oversized prompt file is
  refused with exit 7 naming the path. It is never silently replaced by `--task`'s text.
  `--prompt-file` and `--prompt` together are a usage error.
- Every pre-lane `swarm` failure (`--prompt-file`, `--role`, `--task`, `--agent`, `--repo`)
  now writes a `Refusal` ledger receipt with the process's exit code before exiting
  (AGENTS.md rule 8). Before, none of them wrote one.

### Verification

- Local only (macOS arm64, rustc 1.98.1); no CI.
- New tests: 6 unit + 7 real-binary, all pass; the 8 existing swarm unit tests still pass.
- `cargo test --workspace`: 850 passed, 17 failed; the same 17 fail without this change.
- A 6.8 MB PDF that inflates to 1 GiB: before, the CLI peaked at 1.62 GB RSS over 7.3 s and
  wrote no receipt; after, it is refused in 0.36 s (peak 245 MB, in the child) with a receipt.
- `cargo clippy -D warnings` fails on untouched `crates/plan` with clippy 1.98; 0 warnings in
  changed files.
- Real `fleet swarm --agent freelane --prompt-file <pdf> --merge` built the file the PDF
  described (exit 0).

### Known limitation

- The heap cap and deadline are constants, not configuration.
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

# Changelog

Release convention: every push to `main` requires a matching versioned note under
`docs/releases/` and an entry in this file. See [CONTRIBUTING.md](CONTRIBUTING.md).

## [0.1.3] - 2026-09-28

### Fixed

- `fleet swarm --agent claude` can finish a lane. Before this, every claude lane failed:
  - **Not logged in:** the hermetic env swaps `HOME` and drops `USER`. Fleet now forwards the
    parent's `CLAUDE_CODE_OAUTH_TOKEN` (from `claude setup-token`) to claude lanes only. The
    real `HOME` and `USER` are still never forwarded.
  - **A `---` prompt was an "unknown option":** the prompt is now passed to `claude`/`codex`
    after `--`.
  - **Every write was denied:** swarm workers now run with `--permission-mode acceptEdits`, not
    claude's ask-first default, which has no one to ask in `-p` mode.
  - **The lane's private `HOME` was committed** (session transcript and config) when run with
    `--repo .`. The sandbox path is now absolute, and `merge_lane` never stages
    `.fleet-sandbox/` at any depth.

### Verification

- Local only (macOS arm64, rustc 1.98.1, claude 2.1.280); no CI.
- 9 new tests (5 real-binary, 1 merge integration, 3 unit). Each regression test fails without its fix.
- `cargo test --workspace`: 857 passed, 17 failed; the same 17 fail on untouched `main`.
- Real `fleet swarm --agent claude --prompt-file <.md with front matter> --merge`: exit 0,
  `claude-sonnet-5` created the requested file and fleet merged it.

### Known limitation

- A claude worker can read its own environment, and so the forwarded token. That trade-off is
  for the repo owner to approve, and forwarding stops as soon as the variable is unset.
- The codex `--` change is unverified live. `acceptEdits` also auto-approves simple file
  commands (observed with `touch`).

## [0.1.2] - 2026-09-28

### Added

- `fleet swarm --prompt-file <PATH>` reads the worker instructions from a `.txt`, `.md`, or
  `.pdf` file instead of an inline `--prompt`. PDF text is extracted in-process
  (`pdf-extract`), with no external tool required.

### Fixed

- A prompt starting with `-` (e.g. a plan's `---` front matter) no longer kills the `__agent`
  child with a clap usage error. Before, the lane ended "agent exited without an fd-3 result".

### Changed

- An unsupported, missing, empty, non-UTF-8, textless, malformed, or oversized prompt file is
  refused with exit 7 naming the path. It is never silently replaced by `--task`'s text.
  `--prompt-file` and `--prompt` together are a usage error.

### Verification

- Local only (macOS arm64, rustc 1.98.1); no CI.
- New tests: 6 unit + 5 real-binary, all pass; the 8 existing swarm unit tests still pass.
- `cargo test --workspace`: 848 passed, 17 failed; the same 17 fail on untouched `main`.
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

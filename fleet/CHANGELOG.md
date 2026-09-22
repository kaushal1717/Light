# Changelog

Release convention: every push to `main` requires a matching versioned note under
`docs/releases/` and an entry in this file. See [CONTRIBUTING.md](CONTRIBUTING.md).

## [0.2.1] - 2026-09-22

Independent of 0.2.0 — cut from `main`, no shared files. If this lands first, renumber to 0.2.0.

### Added

- `fleet impact --symbol <name>` now reports the functions that call the symbol, under a new
  `callers` field. `build_repo_map` already returned `edges: Vec<(SymbolId, SymbolId)>`; the
  command loaded them on every run and never read them, so `impact --symbol build` answered `5`
  (five functions *named* `build`) while the repo held 26 places calling one.
- `src/tests/impact_lists_callers` — 5 tests against the real binary.

### Changed

- `ImpactReport` gains `callers`. `matching_symbols` is unchanged, so existing JSON consumers keep
  working.

### Notes

- The field is `callers`, not `call_sites`, because an edge carries no call location: a reported
  line is where the *caller* is defined. A function calling the target three times is one row.
  `the_reported_line_is_the_callers_own_definition_not_the_call` pins that limit on purpose.
- Verified on `posx-mokobara-backend` against grep: `customerIdentityPhone` → 5 call locations
  inside **3** handler functions, reported as 3 callers; `performShopifyReturnWithRefund` → the
  2 subscribers that call it.
- `fleet graph` is untouched and still prints only three counts.

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

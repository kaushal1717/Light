//! A claude lane whose prompt starts with `-` (a `--prompt-file` plan's `---` front matter) must
//! hand that prompt to the `claude` CLI as the prompt. The child used to pass it as a bare argv
//! entry, and the real CLI answered `error: unknown option '---...'`. Drives the REAL binary and
//! `__agent` child; only `claude` is a stub on `PATH` that reports how its last two argv entries
//! arrived and exits 1, so the report comes back in the lane's refusal text. No network.

#[path = "../support/mod.rs"]
mod support;

use std::os::unix::fs::PermissionsExt;
use support::{cmd, scratch_repo};

const STUB: &str = r#"#!/bin/sh
prev=""; last=""
for a in "$@"; do prev="$last"; last="$a"; done
first_line=$(printf '%s' "$last" | head -n 1)
echo "PREV=[$prev] PROMPT_FIRST_LINE=[$first_line]"
exit 1
"#;

#[test]
fn a_front_matter_prompt_reaches_claude_after_end_of_options() {
    let repo = tempfile::tempdir().unwrap();
    scratch_repo(repo.path());
    let bin = tempfile::tempdir().unwrap();
    let stub = bin.path().join("claude");
    std::fs::write(&stub, STUB).unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    let plan = bin.path().join("plan.md");
    std::fs::write(&plan, "---\nplan: EUME-001\n---\n\n# Build the cron\n").unwrap();
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap()
    );
    let out = cmd()
        .env("PATH", path)
        .args([
            "swarm", "--task", "fm", "--role", "builder", "--agent", "claude",
        ])
        .arg("--repo")
        .arg(repo.path())
        .arg("--prompt-file")
        .arg(&plan)
        .output()
        .expect("binary runs");
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        all.contains("PREV=[--]"),
        "prompt not preceded by `--`: {all}"
    );
    assert!(
        all.contains("PROMPT_FIRST_LINE=[---]"),
        "prompt mangled: {all}"
    );
}

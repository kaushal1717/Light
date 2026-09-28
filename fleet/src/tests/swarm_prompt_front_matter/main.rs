//! A prompt that starts with `-` must reach the worker. The parent passes the prompt to the
//! `__agent` child as a positional argv entry, and clap read a leading `---` (YAML front
//! matter, the first line of most `--prompt-file` plans) as a flag: the child died with a usage
//! error (exit 2) before opening fd 3, so every such lane ended "agent exited without an fd-3
//! result". Drives the REAL binary; `FLEET_FREELANE_ROOT` points at a stub so no network is used.

#[path = "../support/mod.rs"]
mod support;

use support::{agent, cmd, scratch_repo, stub_freelane_root};

const FRONT_MATTER: &str = "---\nplan: EUME-001\n---\n\n# Build the STN dispatch cron\n";

#[test]
fn child_accepts_a_task_that_starts_with_a_dash() {
    for task in [FRONT_MATTER, "-v is not a flag here", "--help is text too"] {
        let out = agent(&["freelane", "/tmp", task]);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_ne!(out.status.code(), Some(2), "{task:?} hit clap: {err}");
        assert!(!err.contains("Usage: fleet __agent"), "{task:?}: {err}");
    }
}

#[test]
fn a_front_matter_prompt_file_gets_an_fd3_receipt_back() {
    let repo = tempfile::tempdir().unwrap();
    scratch_repo(repo.path());
    let plan = tempfile::tempdir().unwrap();
    let file = plan.path().join("plan.md");
    std::fs::write(&file, FRONT_MATTER).unwrap();
    let freelane = stub_freelane_root();
    let out = cmd()
        .env("FLEET_FREELANE_ROOT", freelane.path())
        .args(["swarm", "--task", "EUME-001", "--role", "builder", "--repo"])
        .arg(repo.path())
        .arg("--prompt-file")
        .arg(&file)
        .output()
        .expect("binary runs");
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // The stub reports every lane unavailable, so the lane is not `Done`; the property is that
    // the worker parsed its argv and answered over fd 3 instead of dying silently.
    assert!(!all.contains("without an fd-3 result"), "{all}");
    assert!(all.contains("outcome:"), "{all}");
}

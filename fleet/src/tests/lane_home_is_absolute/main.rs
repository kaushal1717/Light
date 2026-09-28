//! `fleet swarm --repo .` must give the worker an ABSOLUTE `HOME`. The worktree path is
//! `repo.join(".worktrees/<name>")`, so a relative `--repo` made the sandbox `HOME` relative too;
//! the `claude` CLI runs with the worktree as its cwd, resolved that path one level deeper
//! (`<worktree>/.worktrees/<name>/.fleet-sandbox/home`), wrote its session files there, and
//! `merge_lane`'s `git add -A` committed them -- the sandbox cleanup only removes the expected
//! `<worktree>/.fleet-sandbox`. Found on a real claude lane. Drives the REAL binary; `claude` is a
//! stub on `PATH` that reports its `HOME` and exits 1. No network.

#[path = "../support/mod.rs"]
mod support;

use std::os::unix::fs::PermissionsExt;
use support::{cmd, scratch_repo};

#[test]
fn a_relative_repo_still_gives_the_worker_an_absolute_home() {
    let repo = tempfile::tempdir().unwrap();
    scratch_repo(repo.path());
    let bin = tempfile::tempdir().unwrap();
    let stub = bin.path().join("claude");
    std::fs::write(&stub, "#!/bin/sh\necho \"HOME=[$HOME]\"\nexit 1\n").unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap()
    );
    let out = cmd()
        .current_dir(repo.path())
        .env("PATH", path)
        .args([
            "swarm",
            "--repo",
            ".",
            "--task",
            "home-probe",
            "--role",
            "builder",
        ])
        .args(["--agent", "claude"])
        .output()
        .expect("binary runs");
    let all = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(all.contains("HOME=["), "stub never ran: {all}");
    assert!(all.contains("HOME=[/"), "worker HOME is relative: {all}");
}

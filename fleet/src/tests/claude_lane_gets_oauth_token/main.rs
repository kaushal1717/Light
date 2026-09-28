//! A real `fleet swarm --agent claude` lane must hand the `claude` CLI the parent's
//! `CLAUDE_CODE_OAUTH_TOKEN`, and nothing that would let it log in the other way (the real
//! `HOME`). Drives the REAL binary and the real `__agent` child; only the `claude` CLI itself is
//! a stub on `PATH`, which prints what it received and exits 1 so the report comes back in the
//! lane's refusal text (`failure_detail`). No network, no real credential.

#[path = "../support/mod.rs"]
mod support;

use std::os::unix::fs::PermissionsExt;
use support::{cmd, scratch_repo};

const TOKEN: &str = "sk-ant-oat01-fleet-test-dummy";

/// Runs one claude lane with the stub first on `PATH`; `token` is the parent's env value.
fn claude_lane(token: Option<&str>) -> String {
    let repo = tempfile::tempdir().unwrap();
    scratch_repo(repo.path());
    let bin = tempfile::tempdir().unwrap();
    let stub = bin.path().join("claude");
    let body = "#!/bin/sh\necho \"TOKEN=[${CLAUDE_CODE_OAUTH_TOKEN:-}] HOME=[$HOME]\"\nexit 1\n";
    std::fs::write(&stub, body).unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!(
        "{}:{}",
        bin.path().display(),
        std::env::var("PATH").unwrap()
    );
    let mut c = cmd();
    c.env("PATH", path).env_remove("CLAUDE_CODE_OAUTH_TOKEN");
    if let Some(t) = token {
        c.env("CLAUDE_CODE_OAUTH_TOKEN", t);
    }
    let out = c
        .args([
            "swarm",
            "--task",
            "token-probe",
            "--role",
            "builder",
            "--agent",
            "claude",
        ])
        .arg("--repo")
        .arg(repo.path())
        .output()
        .expect("binary runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    format!("{stdout}{}", String::from_utf8_lossy(&out.stderr))
}

#[test]
fn a_claude_lane_receives_the_parents_token() {
    let all = claude_lane(Some(TOKEN));
    assert!(all.contains(&format!("TOKEN=[{TOKEN}]")), "{all}");
}

#[test]
fn no_token_in_the_parent_means_none_in_the_lane() {
    let all = claude_lane(None);
    assert!(all.contains("TOKEN=[]"), "{all}");
}

#[test]
fn the_token_does_not_bring_the_real_home_with_it() {
    let all = claude_lane(Some(TOKEN));
    let real_home = std::env::var("HOME").unwrap();
    assert!(all.contains("HOME=["), "stub never ran: {all}");
    assert!(!all.contains(&format!("HOME=[{real_home}]")), "{all}");
}

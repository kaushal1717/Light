//! `fleet swarm --prompt-file` against the REAL compiled binary (`env!("CARGO_BIN_EXE_fleet")`).
//! Offline and deterministic: every call passes `--agent bogus`, which `swarm` refuses as an
//! EnvironmentFault (3) only AFTER the prompt is resolved. So exit 3 proves the file was read
//! and accepted, while a bad file must stop earlier with a Refusal (7) naming `--prompt-file`.

#[path = "bomb.rs"]
mod bomb;
#[path = "expansion.rs"]
mod expansion;
#[path = "../../dispatch/run/swarm_prompt_file_fixture.rs"]
mod fixture;
#[path = "receipts.rs"]
mod receipts;
#[path = "../support/mod.rs"]
mod support;

use support::cmd;

const ENV_FAULT: i32 = 3;
const REFUSAL: i32 = 7;
pub struct Swarm {
    pub code: Option<i32>,
    pub stderr: String,
    pub state: tempfile::TempDir,
}

/// `swarm --prompt-file <name>` (`bytes` written unless `None`) in its own FLEET_STATE_DIR.
pub fn swarm_with(name: &str, bytes: Option<&[u8]>, extra: &[&str]) -> Swarm {
    let (dir, state) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let file = dir.path().join(name);
    if let Some(bytes) = bytes {
        std::fs::write(&file, bytes).unwrap();
    }
    let (repo, file) = (dir.path().to_string_lossy(), file.to_string_lossy());
    let out = cmd()
        .env("FLEET_STATE_DIR", state.path())
        .args([
            "swarm", "--repo", &repo, "--task", "EUME-001", "--role", "builder",
        ])
        .args(["--agent", "bogus", "--prompt-file", &file])
        .args(extra)
        .output()
        .expect("binary runs");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    Swarm {
        code: out.status.code(),
        stderr,
        state,
    }
}

#[test]
fn md_txt_and_pdf_files_are_accepted_as_the_prompt() {
    let pdf = fixture::one_page_pdf("Build the STN dispatch cron");
    let md = b"# EUME-001\nBuild the cron.";
    for (name, bytes) in [
        ("plan.md", &md[..]),
        ("plan.txt", b"Build."),
        ("plan.pdf", &pdf),
    ] {
        let run = swarm_with(name, Some(bytes), &[]);
        let err = &run.stderr;
        assert_eq!(run.code, Some(ENV_FAULT), "{name}: {err}");
        // Got past the prompt and stopped at `--agent`, not at the file.
        assert!(err.contains("--agent"), "{name}: {err}");
        assert!(!err.contains("--prompt-file"), "{name}: {err}");
    }
}

#[test]
fn prompt_and_prompt_file_together_are_a_usage_error() {
    let run = swarm_with("plan.md", Some(b"Build the cron."), &["--prompt", "inline"]);
    assert_ne!(
        run.code,
        Some(0),
        "both flags must not run a lane: {}",
        run.stderr
    );
    assert!(run.stderr.contains("cannot be used with"), "{}", run.stderr);
}

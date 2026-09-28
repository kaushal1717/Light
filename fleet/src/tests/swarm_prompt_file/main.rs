//! `fleet swarm --prompt-file` against the REAL compiled binary (`env!("CARGO_BIN_EXE_fleet")`).
//! Offline and deterministic: every call passes `--agent bogus`, which `swarm` refuses as an
//! EnvironmentFault (3) only AFTER the prompt is resolved. So exit 3 proves the file was read
//! and accepted, while a bad file must stop earlier with a Refusal (7) naming `--prompt-file`.

#[path = "../../dispatch/run/swarm_prompt_file_fixture.rs"]
mod fixture;
#[path = "../support/mod.rs"]
mod support;

use support::cmd;

const ENV_FAULT: i32 = 3;
const REFUSAL: i32 = 7;

/// Runs `swarm` with `bytes` written to `name`, returning `(exit code, stderr)`.
fn swarm_with(name: &str, bytes: &[u8], extra: &[&str]) -> (Option<i32>, String) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join(name);
    std::fs::write(&file, bytes).unwrap();
    let repo = dir.path().to_string_lossy().into_owned();
    let file = file.to_string_lossy().into_owned();
    let out = cmd()
        .args(["swarm", "--repo", &repo, "--task", "EUME-001"])
        .args([
            "--role",
            "builder",
            "--agent",
            "bogus",
            "--prompt-file",
            &file,
        ])
        .args(extra)
        .output()
        .expect("binary runs");
    let err = String::from_utf8_lossy(&out.stderr).into_owned();
    (out.status.code(), err)
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
        let (code, err) = swarm_with(name, bytes, &[]);
        assert_eq!(code, Some(ENV_FAULT), "{name}: {err}");
        // Got past the prompt and stopped at `--agent`, not at the file.
        assert!(err.contains("--agent"), "{name}: {err}");
        assert!(!err.contains("--prompt-file"), "{name}: {err}");
    }
}

#[test]
fn unsupported_and_empty_files_are_refused_with_exit_7() {
    for (name, bytes, why) in [
        (
            "plan.docx",
            &b"Build the cron."[..],
            "unsupported file type",
        ),
        ("blank.md", &b"  \n"[..], "empty"),
    ] {
        let (code, err) = swarm_with(name, bytes, &[]);
        assert_eq!(code, Some(REFUSAL), "{name}: {err}");
        assert!(err.contains("--prompt-file"), "{name}: {err}");
        assert!(err.contains(why), "{name}: {err}");
    }
}

#[test]
fn prompt_and_prompt_file_together_are_a_usage_error() {
    let (code, err) = swarm_with("plan.md", b"Build the cron.", &["--prompt", "inline"]);
    assert_ne!(code, Some(0), "both flags must not run a lane: {err}");
    assert!(err.contains("cannot be used with"), "{err}");
}

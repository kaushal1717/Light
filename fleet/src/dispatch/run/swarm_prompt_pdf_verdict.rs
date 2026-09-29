//! What the `__pdf-text` child's exit means. Exit 7 carries the child's own refusal reason; an
//! abort after a failed allocation is the heap cap doing its job; a timeout is the deadline's.
//! Only a child that could not run at all (any other exit code) is an environment fault.

use super::pdf_text_cmd::HEAP_CAP_BYTES;
use super::swarm_prompt_file::Failure;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::time::Duration;

pub(super) fn verdict(
    status: Option<ExitStatus>,
    deadline: Duration,
    out: Vec<u8>,
    err: &str,
) -> Result<String, Failure> {
    let Some(status) = status else {
        let secs = deadline.as_secs();
        return Err(format!("PDF text extraction exceeded its {secs}s time limit").into());
    };
    let reason = err
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    let reason = reason.trim().trim_start_matches("fleet: ").to_string();
    match (status.code(), status.signal()) {
        (Some(0), _) => {
            String::from_utf8(out).map_err(|_| "PDF text is not UTF-8".to_string().into())
        }
        (Some(7), _) => Err(Failure::Refused(reason)),
        // Rust reports a failed allocation as this line on stderr, then aborts.
        (None, _) if err.contains("memory allocation of") => Err(format!(
            "PDF needs more than {} MiB of memory to extract (its streams inflate too far)",
            HEAP_CAP_BYTES >> 20
        )
        .into()),
        (None, signal) => Err(format!("PDF parser crashed (signal {signal:?})").into()),
        (Some(code), _) => Err(Failure::Env(format!("PDF parser exited {code}: {reason}"))),
    }
}

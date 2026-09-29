//! Every `--prompt-file` refusal leaves a durable `Refusal` receipt before the process exits
//! (AGENTS.md rule 8), carrying the same typed exit code the process returned.

use super::{fixture, swarm_with, Swarm, ENV_FAULT, REFUSAL};
use store::ledger::LedgerPaths;
use store::Ledger;
use types::{ExitCode, ReceiptEvent};

/// The one receipt the run wrote: a `Refusal` carrying the same exit code the process returned.
pub fn only_refusal_receipt(run: &Swarm, code: ExitCode) -> serde_json::Value {
    let ledger = Ledger::open(LedgerPaths {
        chain: run.state.path().join("ledger.chain"),
        lock: run.state.path().join("ledger.lock"),
    });
    let rows = ledger
        .rows(true)
        .expect("a verifiable ledger with receipt rows");
    assert_eq!(rows.len(), 1, "exactly one receipt: {rows:?}");
    assert_eq!(rows[0].event, ReceiptEvent::Refusal);
    assert_eq!(rows[0].exit_code, Some(code));
    assert_eq!(rows[0].body["outcome"], "refused");
    rows[0].body.clone()
}

#[test]
fn missing_unsupported_and_empty_files_exit_7_with_a_refusal_receipt() {
    let textless = fixture::one_page_pdf("");
    for (name, bytes, why) in [
        ("absent.md", None, "absent.md"),
        (
            "plan.docx",
            Some(&b"Build the cron."[..]),
            "unsupported file type",
        ),
        ("blank.md", Some(&b"  \n"[..]), "empty"),
        ("scan.pdf", Some(&textless[..]), "no extractable text"),
        (
            "junk.pdf",
            Some(&b"%PDF-1.4 not really a pdf"[..]),
            "could not read PDF text",
        ),
    ] {
        let run = swarm_with(name, bytes, &[]);
        let err = &run.stderr;
        assert_eq!(run.code, Some(REFUSAL), "{name}: {err}");
        assert!(
            err.contains("--prompt-file") && err.contains(why),
            "{name}: {err}"
        );
        let body = only_refusal_receipt(&run, ExitCode::Refusal);
        assert!(
            body["failure"].as_str().unwrap().contains(why),
            "{name}: {body}"
        );
        assert!(
            body["prompt_file"].as_str().unwrap().ends_with(name),
            "{name}: {body}"
        );
    }
}

#[test]
fn an_accepted_file_that_stops_at_intake_still_leaves_a_receipt() {
    let run = swarm_with("plan.md", Some(b"Build the cron."), &[]);
    assert_eq!(run.code, Some(ENV_FAULT), "{}", run.stderr);
    let body = only_refusal_receipt(&run, ExitCode::Env);
    assert!(
        body["failure"].as_str().unwrap().contains("--agent"),
        "{body}"
    );
}

//! A PDF whose streams inflate past the parser's heap cap is refused by the isolated
//! `__pdf-text` child's limit -- fast, with a receipt -- never by the CLI running out of memory.
//! Before the child existed, this 6.8 MB file drove the CLI itself to a 1.6 GB peak RSS.

use super::receipts::only_refusal_receipt;
use super::{bomb, swarm_with, REFUSAL};
use types::ExitCode;

#[test]
fn a_pdf_that_inflates_past_the_heap_cap_is_refused_by_the_isolated_child() {
    let pdf = bomb::expansion_pdf();
    assert!(
        pdf.len() < 16 * 1024 * 1024,
        "the fixture must pass the on-disk size check"
    );
    let started = std::time::Instant::now();
    let run = swarm_with("bomb.pdf", Some(&pdf), &[]);
    let err = &run.stderr;
    assert_eq!(run.code, Some(REFUSAL), "{err}");
    assert!(err.contains("MiB of memory"), "{err}");
    assert!(
        started.elapsed().as_secs() < 30,
        "bounded, not a wait for 1 GiB to inflate"
    );
    only_refusal_receipt(&run, ExitCode::Refusal);
}

use super::{read, MAX_PROMPT_BYTES};
use crate::dispatch::error::DispatchError;
use std::path::PathBuf;

#[path = "swarm_prompt_file_fixture.rs"]
mod fixture;

fn write(dir: &tempfile::TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

fn refusal(path: &std::path::Path) -> String {
    match read(path) {
        Err(DispatchError::Refusal(reason)) => reason,
        other => panic!("expected a Refusal for {}, got {other:?}", path.display()),
    }
}

#[test]
fn txt_and_md_are_passed_through_verbatim() {
    let dir = tempfile::tempdir().unwrap();
    let body = "# EUME-001\n\nBuild the STN dispatch cron.\n";
    for name in ["plan.txt", "plan.md", "PLAN.MD"] {
        assert_eq!(read(&write(&dir, name, body.as_bytes())).unwrap(), body);
    }
}

#[test]
fn pdf_text_layer_becomes_the_prompt() {
    let dir = tempfile::tempdir().unwrap();
    let pdf = fixture::one_page_pdf("Build the STN dispatch cron");
    let text = read(&write(&dir, "plan.pdf", &pdf)).unwrap();
    assert!(text.contains("Build the STN dispatch cron"), "got {text:?}");
}

#[test]
fn unsupported_or_missing_extension_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["plan.docx", "plan"] {
        let reason = refusal(&write(&dir, name, b"real instructions"));
        assert!(reason.contains("unsupported file type"), "{name}: {reason}");
    }
}

#[test]
fn missing_empty_and_non_utf8_files_are_refused_not_defaulted() {
    let dir = tempfile::tempdir().unwrap();
    assert!(refusal(&dir.path().join("absent.md")).contains("absent.md"));
    assert!(refusal(&write(&dir, "blank.md", b" \n\t\n")).contains("empty"));
    assert!(refusal(&write(&dir, "bin.txt", &[0xff, 0xfe, 0x00])).contains("UTF-8"));
}

#[test]
fn malformed_and_textless_pdfs_are_refused_not_panicked() {
    let dir = tempfile::tempdir().unwrap();
    let garbage = refusal(&write(&dir, "junk.pdf", b"%PDF-1.4 not really a pdf"));
    assert!(garbage.contains("could not read PDF text"), "{garbage}");
    let blank = refusal(&write(&dir, "blank.pdf", &fixture::one_page_pdf("")));
    assert!(blank.contains("no extractable text"), "{blank}");
}

#[test]
fn text_too_large_for_one_argv_entry_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let big = "a".repeat(MAX_PROMPT_BYTES + 1);
    assert!(refusal(&write(&dir, "big.txt", big.as_bytes())).contains("limit"));
}

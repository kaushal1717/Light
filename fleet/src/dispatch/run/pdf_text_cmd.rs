//! `fleet __pdf-text`: the isolated child `swarm --prompt-file` runs every PDF through -- the PDF
//! on stdin, its text layer on stdout, a refusal reason on stderr with exit 7. The parser runs
//! HERE, never in the CLI: a PDF under the 16 MiB file limit can declare streams (content, object,
//! or xref streams) that inflate to gigabytes before any text-length check can run, and
//! `pdf-extract`/`lopdf` expose no decompression limit. So this process arms a heap cap
//! (`alloc_cap`) and the parent kills it at a wall-clock deadline (`swarm_prompt_pdf.rs`).

use super::swarm_prompt_file::{MAX_FILE_BYTES, MAX_PROMPT_BYTES};
use crate::dispatch::error::DispatchError;
use std::io::{Read, Write};

/// Net heap the parser may grow by: room for any text PDF's fonts and page streams, and a small
/// fraction of what an expansion bomb asks for. Crossing it aborts this child.
pub(crate) const HEAP_CAP_BYTES: usize = 256 * 1024 * 1024;

pub fn run() -> Result<(), DispatchError> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| DispatchError::EnvFault(format!("PDF on stdin: {e}")))?;
    crate::alloc_cap::arm(HEAP_CAP_BYTES);
    let text = extract(&bytes).map_err(DispatchError::Refusal)?;
    std::io::stdout()
        .write_all(text.as_bytes())
        .map_err(|e| DispatchError::EnvFault(format!("PDF text on stdout: {e}")))
}

/// `pdf-extract` can panic on malformed input rather than return `Err`; a bad file the dev
/// pointed at must still end as a typed refusal. The length check runs here, before anything
/// is written, so the parent's bounded read of stdout never has to stall an oversized child.
pub(crate) fn extract(bytes: &[u8]) -> Result<String, String> {
    let text = match std::panic::catch_unwind(|| pdf_extract::extract_text_from_mem(bytes)) {
        Ok(Ok(text)) => text,
        Ok(Err(e)) => return Err(format!("could not read PDF text: {e}")),
        Err(_) => return Err("could not read PDF text: the PDF parser rejected the file".into()),
    };
    if text.len() > MAX_PROMPT_BYTES {
        return Err(format!(
            "text is {} bytes (limit {MAX_PROMPT_BYTES}); split it into smaller lanes",
            text.len()
        ));
    }
    Ok(text)
}

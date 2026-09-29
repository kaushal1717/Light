//! `fleet swarm --prompt-file`: the worker instructions read from a `.txt`, `.md`, or `.pdf`
//! file instead of an inline `--prompt`. Every way the file can fail to yield usable text is a
//! `Refusal` (exit 7) naming the path -- never a silent fallback to `--task`'s text, which would
//! send the worker a task id in place of the instructions the dev pointed at. A PDF is parsed in
//! an isolated, resource-capped child (`swarm_prompt_pdf.rs`), never in this process.

use crate::dispatch::error::DispatchError;
use std::path::Path;

/// Bound on the bytes read from disk, so a mistyped path to a huge file fails fast.
pub(crate) const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
/// The prompt reaches the worker as ONE argv entry (`builder`'s `child_command.rs`), and macOS
/// caps argv + environment at 1 MiB (`ARG_MAX`). A quarter of that leaves room for the rest.
pub(crate) const MAX_PROMPT_BYTES: usize = 256 * 1024;

#[cfg(test)]
#[path = "swarm_prompt_file_tests.rs"]
mod tests;

enum Kind {
    Text,
    Pdf,
}
/// The file's fault (exit 7), or this machine's (exit 3), e.g. a PDF parser that can't start.
pub(crate) enum Failure {
    Refused(String),
    Env(String),
}

impl From<String> for Failure {
    fn from(reason: String) -> Self {
        Failure::Refused(reason)
    }
}

pub(crate) fn read(path: &Path) -> Result<String, DispatchError> {
    let at = |reason| format!("--prompt-file {}: {reason}", path.display());
    read_text(path).map_err(|failure| match failure {
        Failure::Refused(reason) => DispatchError::Refusal(at(reason)),
        Failure::Env(reason) => DispatchError::EnvFault(at(reason)),
    })
}

fn kind(path: &Path) -> Result<Kind, String> {
    let ext = path.extension().and_then(|e| e.to_str());
    match ext.map(str::to_ascii_lowercase).as_deref() {
        Some("txt" | "md") => Ok(Kind::Text),
        Some("pdf") => Ok(Kind::Pdf),
        _ => Err("unsupported file type (expected .txt, .md, or .pdf)".into()),
    }
}

fn read_text(path: &Path) -> Result<String, Failure> {
    let kind = kind(path)?;
    let len = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if len > MAX_FILE_BYTES {
        return Err(format!("file is {len} bytes (limit {MAX_FILE_BYTES})").into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let text = match kind {
        Kind::Text => {
            String::from_utf8(bytes).map_err(|_| "file is not valid UTF-8".to_string())?
        }
        Kind::Pdf => super::swarm_prompt_pdf::text(&bytes)?,
    };
    if text.trim().is_empty() {
        return Err(match kind {
            Kind::Text => "file is empty or all whitespace".to_string(),
            Kind::Pdf => "PDF has no extractable text (a scanned PDF has no text layer)".into(),
        }
        .into());
    }
    if text.len() > MAX_PROMPT_BYTES {
        let len = text.len();
        let why =
            format!("text is {len} bytes (limit {MAX_PROMPT_BYTES}); split it into smaller lanes");
        return Err(why.into());
    }
    Ok(text)
}

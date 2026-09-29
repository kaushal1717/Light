//! The `Refusal` receipt `swarm_intake` writes when `fleet swarm` stops before a lane exists.

use crate::dispatch::error::DispatchError;
use cli::args_core::SwarmArgs;
use serde_json::json;
use std::path::Path;
use types::ReceiptEvent;

/// The receipt carries the typed exit code, so the ledger and the process can never disagree.
/// Failing to write it is itself an environment fault: a refusal with no receipt is the exact
/// gap rule 8 exists to close, so it is surfaced rather than swallowed.
pub(crate) fn refusal(
    state_dir: &Path,
    args: &SwarmArgs,
    error: &DispatchError,
) -> Result<(), DispatchError> {
    let body = json!({
        "outcome": "refused",
        "stage": "swarm-intake",
        "task": args.task,
        "prompt_file": args.prompt_file,
        "failure": error.to_string(),
        "checked": 0,
        "total": 0
    });
    std::fs::create_dir_all(state_dir)
        .map_err(|e| e.to_string())
        .and_then(|()| {
            crate::pipeline::ledger_events::append_as_with_exit(
                state_dir,
                ReceiptEvent::Refusal,
                body,
                "fleet-cli-swarm",
                Some(error.exit_code()),
            )
        })
        .map_err(|e| DispatchError::EnvFault(format!("swarm refusal receipt: {e}")))
}

//! `fleet swarm` intake: every check that can stop a swarm BEFORE a lane exists. A lane writes
//! its own receipts once spawned; nothing here has a lane yet, so a failure here writes its own
//! `Refusal` receipt before `swarm` exits (AGENTS.md rule 8) -- a bad `--prompt-file`, `--role`,
//! `--task`, `--agent`, or `--repo` is never an exit code with no durable record behind it.

use crate::dispatch::error::DispatchError;
use builder::CliAdapter;
use cli::args_core::SwarmArgs;
use std::path::{Path, PathBuf};
use types::{Role, TaskId};

pub(crate) struct Intake {
    pub role: Role,
    pub task_id: TaskId,
    pub prompt: String,
    pub adapter: CliAdapter,
    pub repo: PathBuf,
}

pub(crate) fn resolve(state_dir: &Path, args: &SwarmArgs) -> Result<Intake, DispatchError> {
    parse(args).or_else(|error| {
        super::swarm_intake_receipt::refusal(state_dir, args, &error)?;
        Err(error)
    })
}

fn parse(args: &SwarmArgs) -> Result<Intake, DispatchError> {
    // Cheap, offline validation first -- fail fast before ever touching the filesystem
    // (`ensure_repo` below, which walks `--repo`) or the network. An invalid `--role`/`--agent`
    // must refuse the same way regardless of whether `--repo` happens to exist (see the
    // ordering-sensitive regression tests in swarm_cmd_tests.rs).
    let role = Role::parse(&args.role).map_err(|e| DispatchError::Refusal(e.to_string()))?;
    let task_id =
        TaskId::parse(args.task.clone()).map_err(|e| DispatchError::Refusal(e.to_string()))?;
    // `--task` is both the lane's task id and, unless `--prompt` overrides it, the free-text
    // instructions sent to the worker (S1-4). `--prompt-file` (clap-exclusive with `--prompt`)
    // reads them from disk; a file that yields no usable text is refused, never silently
    // replaced by `--task`'s text.
    let prompt = match &args.prompt_file {
        Some(path) => crate::dispatch::swarm_prompt_file::read(Path::new(path))?,
        None if args.prompt.trim().is_empty() => args.task.clone(),
        None => args.prompt.clone(),
    };
    // `--agent` selects the CLI adapter; unknown values are an EnvironmentFault, never a silent
    // default to Freelane. `from_agent_kind` is the single parse point.
    let adapter = CliAdapter::from_agent_kind(&args.agent)
        .map_err(|e| DispatchError::EnvFault(format!("--agent {:?}: {e}", args.agent)))?;
    // A `--repo` that isn't a git worktree is refused up front with an actionable message; the
    // lane's worktree/checkout/merge stages all assume one and fail opaquely otherwise.
    let repo = super::verify_repo::ensure_repo(&args.repo)?;
    Ok(Intake {
        role,
        task_id,
        prompt,
        adapter,
        repo,
    })
}

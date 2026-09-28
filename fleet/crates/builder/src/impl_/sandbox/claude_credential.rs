//! The ONE credential a hermetic lane may receive: `CLAUDE_CODE_OAUTH_TOKEN` (minted by
//! `claude setup-token`), and only by a `claude` lane. The hermetic env swaps `HOME` for a
//! per-lane tempdir and drops `USER`, so the macOS `claude` CLI can never find its Keychain login
//! (every claude lane ended "Not logged in"). This token lets it log in WITHOUT the user's real
//! `~/.claude`. It is never forwarded to codex/freelane lanes, never stored in `HermeticEnv`
//! (whose `Debug` could be logged), and only read from the parent's env, never from config.

use super::super::adapter::CliAdapter;
use std::process::Command;

pub const ENV_CLAUDE_OAUTH_TOKEN: &str = "CLAUDE_CODE_OAUTH_TOKEN";

/// Set the token on `command` if (and only if) `adapter` is `claude` and the parent has one.
/// Must run AFTER `HermeticEnv::apply`, whose `env_clear()` would otherwise wipe it.
pub fn forward(adapter: CliAdapter, command: &mut Command) {
    if let Some(token) = token_for(adapter, std::env::var(ENV_CLAUDE_OAUTH_TOKEN).ok()) {
        command.env(ENV_CLAUDE_OAUTH_TOKEN, token);
    }
}

/// Pure decision, so the allowlist is testable without mutating the process env.
fn token_for(adapter: CliAdapter, parent: Option<String>) -> Option<String> {
    match adapter {
        CliAdapter::Claude => parent.filter(|t| !t.trim().is_empty()),
        CliAdapter::Codex | CliAdapter::Freelane => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok() -> Option<String> {
        Some("sk-ant-oat01-test".into())
    }

    #[test]
    fn only_a_claude_lane_gets_the_token() {
        assert_eq!(token_for(CliAdapter::Claude, tok()), tok());
        assert_eq!(token_for(CliAdapter::Codex, tok()), None);
        assert_eq!(token_for(CliAdapter::Freelane, tok()), None);
    }

    #[test]
    fn an_absent_or_blank_token_is_not_forwarded() {
        assert_eq!(token_for(CliAdapter::Claude, None), None);
        assert_eq!(token_for(CliAdapter::Claude, Some("  ".into())), None);
    }
}

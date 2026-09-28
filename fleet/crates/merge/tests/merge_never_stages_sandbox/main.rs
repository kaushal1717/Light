//! `merge_lane` must never commit a lane's private `HOME` (`.fleet-sandbox/`), at any depth. A
//! real claude lane run with `--repo .` wrote its session transcript and config to a nested
//! `.worktrees/<name>/.fleet-sandbox/home/` inside the worktree, and `git add -A` committed six of
//! those files next to the one real change.

#[path = "../common/mod.rs"]
mod common;

use common::init_repo;
use merge::{create, merge_lane, remove};

#[test]
fn sandbox_files_at_any_depth_are_left_out_of_the_lane_commit() {
    let (_guard, repo) = init_repo();
    let wt = create(&repo, &merge::unique_name("sandbox")).expect("create");
    let nested = wt.path.join(".worktrees/x/.fleet-sandbox/home/.claude");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(nested.join("session.jsonl"), b"transcript").unwrap();
    std::fs::create_dir_all(wt.path.join(".fleet-sandbox/home")).unwrap();
    std::fs::write(wt.path.join(".fleet-sandbox/home/.claude.json"), b"{}").unwrap();
    std::fs::write(wt.path.join("service.ts"), b"export {}").unwrap();

    let outcome = merge_lane(&repo, &wt.path, &wt.branch).expect("merge_lane");
    assert_eq!(outcome.staged_files, 1, "only the real change is staged");
    assert!(repo.join("service.ts").exists());
    assert!(!repo.join(".fleet-sandbox").exists());
    assert!(!repo.join(".worktrees/x").exists());

    remove(&repo, &wt).expect("cleanup");
}

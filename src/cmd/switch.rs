use std::path::Path;

use crate::{
    error::GitJumpError, git::git_command, storage::update_branch_last_switch, types::now,
};

/// Switches to `target` via `git switch` and records the jump timestamp.
///
/// On success, resolves the checked-out branch name with `rev-parse`
/// and attempts to update the jump data file.
/// If `rev-parse` fails, the switch still succeeds.
///
/// side-effects: executes `git switch`, updates jump data
pub fn switch_and_record(data_file: &Path, target: &str) -> Result<String, GitJumpError> {
    record_switch(data_file, git_command("switch", &[target]))
}

/// side-effects: executes `git switch`, updates jump data
pub fn switch_to_remote_and_record(
    data_file: &Path,
    branch_name: &str,
    remote: &str,
) -> Result<String, GitJumpError> {
    let remote_ref = format!("{remote}/{branch_name}");
    record_switch(data_file, git_command("switch", &["--track", &remote_ref]))
}

fn record_switch(
    data_file: &Path,
    switch_result: anyhow::Result<String>,
) -> Result<String, GitJumpError> {
    match switch_result {
        Ok(msg) => {
            if let Ok(branch_name) = git_command("rev-parse", &["--abbrev-ref", "HEAD"]) {
                update_branch_last_switch(data_file, &branch_name, now())?;
            }
            Ok(msg)
        }
        Err(err) => Err(GitJumpError::SwitchFailed(err)),
    }
}

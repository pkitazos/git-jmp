use crate::{
    branch::{generate_ranked_list, get_active_worktree},
    cmd::{Run, switch::switch_and_record},
    error::GitJumpError,
    model::Model,
};

pub struct JumpTo {
    pub branch: String,
}

impl Run for JumpTo {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        jump_to(state, &self.branch).map(|res| {
            println!("{res}");
        })
    }
}

/// Jumps to the best matching branch.
///
/// - Short-circuits if the target is the active branch.
/// - `-` is treated as jump to the previous branch
/// - attempts exact match first
/// - falls back to fuzzy match
///
/// side-effect: executes `git switch`, updates jump data
fn jump_to(state: &Model, target: &str) -> Result<String, GitJumpError> {
    let current_worktree = get_active_worktree(&state.worktrees, &state.active_worktree);

    if target.eq(current_worktree.head.label()) {
        return Ok(format!("Staying on {}", target));
    }

    if target == "-" {
        return switch_and_record(&state.main_worktree.data_file(), "-");
    }

    match switch_and_record(&state.main_worktree.data_file(), target) {
        Ok(msg) => return Ok(msg),
        Err(e) => {
            if state.branches.iter().any(|b| b.name == target) {
                return Err(e);
            }
        }
    };

    let list = generate_ranked_list(&state.branches, &state.worktrees, target);

    if list.available.is_empty() {
        return Err(GitJumpError::NoMatch {
            target: target.to_string(),
        });
    }

    switch_and_record(&state.main_worktree.data_file(), &list.available[0].name)
}

use clap::Parser;

use crate::{
    cmd::Run,
    git::git_command,
    list::get_active_worktree,
    storage::rename_jump_data_branch,
    types::{GitJumpError, Head, Model},
};

#[derive(Debug, Parser)]
/// Rename branch called <curr_name> to <new_name>
#[command(
    visible_alias("mv"),
    arg_required_else_help = true,
    override_usage = "git-jump rename [CURRENT_NAME] <NEW_NAME>"
)]
pub struct Rename {
    #[arg(num_args = 1..=2)]
    names: Vec<String>,
}

impl Run for Rename {
    fn run(&self, state: &Model) -> Result<(), GitJumpError> {
        let (current_name, new_name) = match self.names.as_slice() {
            [new_name] => (None, new_name),
            [current_name, new_name] => (Some(current_name.as_str()), new_name),
            _ => unreachable!("clap grammar prevents this"),
        };

        rename_sub_command(state, current_name, new_name).map(|res| {
            println!("{res}");
        })
    }
}

/// side-effect: update the JumpData file
pub fn rename_sub_command(
    state: &Model,
    src: Option<&str>,
    target: &str,
) -> Result<String, GitJumpError> {
    let src = match src {
        Some(name) => name.to_owned(),
        None => match get_active_worktree(&state.worktrees, &state.active_worktree).head {
            Head::Branch(b) => b.name.clone(),
            Head::Detached { .. } => return Err(GitJumpError::DetachedHead),
        },
    };

    match git_command("branch", &["--move", &src, target]) {
        Ok(msg) => {
            rename_jump_data_branch(&state.main_worktree.data_file(), &src, target)?;
            Ok(msg)
        }
        Err(err) => Err(GitJumpError::BranchRenaming(err)),
    }
}

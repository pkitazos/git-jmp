use clap::Parser;

use crate::{
    branch::get_active_worktree, cmd::Run, error::GitJumpError, git::git_command, model::Model,
    storage::rename_jump_data_branch, types::Head,
};

#[derive(Debug, Parser)]
/// Rename a branch
///
/// Runs `git branch --move` under the hood and updates your jump data.
#[command(
    arg_required_else_help = true,
    override_usage = "git jmp mv <NEW_NAME>\n       git jmp mv <CURRENT_NAME> <NEW_NAME>",
    help_template = "\
{about-with-newline}
{usage-heading} {usage}

Arguments:
  <CURRENT_NAME>  Branch to rename (defaults to the current branch
                  when only one argument is given)
  <NEW_NAME>      New name for the branch

Options:
{options}{after-help}"
)]
pub struct Rename {
    #[arg(num_args = 1..=2, value_name = "NAMES", hide = true)]
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

use crate::types::{BranchDeleteResult, GitJumpError};

// const HEAD_INDEX_PADD: &'static str = " * ";
pub const BRANCH_INDEX_PADD: &'static str = "   ";
// const LINE_SPACER: &'static str = "  ";

pub fn render_git_jump_error(err: GitJumpError) {
    match err {
        GitJumpError::BranchCreation(error) => todo!(),
        GitJumpError::BranchRenaming(error) => todo!(),
        GitJumpError::NoMatch { target } => todo!(),
        GitJumpError::SwitchFailed(error) => todo!(),
        GitJumpError::DetachedHead => todo!(),
        GitJumpError::Other(error) => todo!(),
    }
}

pub fn render_branch_deletion_res(res: Vec<BranchDeleteResult>) {}

pub fn render_branch_list(branches: Vec<String>) {}

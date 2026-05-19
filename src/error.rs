use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitJumpError {
    #[error("Failed to create branch")]
    BranchCreation(#[source] anyhow::Error),

    #[error("Failed to rename branch")]
    BranchRenaming(#[source] anyhow::Error),

    #[error("{target} does not match any branch")]
    NoMatch { target: String },

    #[error("Failed to switch branch")]
    SwitchFailed(#[source] anyhow::Error),

    #[error("Can't rename: HEAD is detached, specify the branch explicitly")]
    DetachedHead,

    #[error("")]
    SilentExit,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

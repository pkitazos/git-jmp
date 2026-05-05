use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use git_jump::{
    command::{delete_sub_command, jump_to, list_sub_command, new_sub_command, rename_sub_command},
    git::{GitDirs, list_worktrees, locate_git_repo_dirs, read_raw_git_branches},
    storage::get_and_clean_branches,
    types::{Branch, Model, ModifierKey, Msg, Worktree},
};

#[derive(Parser)]
#[command(name = "git-jump")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    pub branch: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List all branches
    #[clap(visible_alias("ls"))] // not sure how to actually use these?
    List,

    #[command(arg_required_else_help = true)]
    /// Create a new branch called <branch_name>
    New { branch_name: String },

    #[clap(visible_alias("rm"))]
    #[command(arg_required_else_help = true)]
    /// Delete listed branches
    Delete {
        #[arg(num_args = 1..)]
        branch_names: Vec<String>,
    },

    #[clap(visible_alias("mv"))]
    #[command(arg_required_else_help = true)]
    /// Rename branch called <curr_name> to <new_name>
    Rename {
        current_name: String,
        new_name: String,
    },
}

enum Invocation {
    Interactive,
    JumpTo(String),
    Sub(Commands),
}

impl Cli {
    fn into_invocation(self) -> Invocation {
        match (self.command, self.branch) {
            (Some(c), None) => Invocation::Sub(c),
            (None, Some(b)) => Invocation::JumpTo(b),
            (None, None) => Invocation::Interactive,
            (Some(_), Some(_)) => unreachable!("clap grammar prevents this"),
        }
    }
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    let data = init()?;

    let (columns, rows) = crossterm::terminal::size()?;

    let modifier_key = if std::env::consts::OS == "macos" {
        ModifierKey::Option
    } else {
        ModifierKey::Alt
    };

    let state: Model = Model {
        main_worktree: data.main_worktree,
        active_worktree: data.active_worktree,
        columns: columns as usize,
        rows: rows as usize,
        max_rows: rows as usize,
        branches: data.branches,
        worktrees: data.worktrees,
        modifier_key: modifier_key,
        interactive_state: None,
    };

    match &cli.into_invocation() {
        Invocation::Interactive => todo!(),
        Invocation::JumpTo(branch) => {
            let _ = jump_to(&state, branch, &[])?;
            // todo: render
        }
        Invocation::Sub(commands) => {
            let _ = dispatch_sub_command(&state, commands)?;
            // todo: render
        }
    }

    Ok(())
}

fn dispatch_sub_command(state: &Model, cmd: &Commands) -> Result<Msg> {
    match cmd {
        Commands::List => {
            println!("[LIST]");
            list_sub_command(state)
        }

        Commands::New { branch_name } => {
            println!("[NEW]");
            new_sub_command(state, branch_name)
        }

        Commands::Delete { branch_names } => {
            println!("[DELETE]");
            delete_sub_command(
                state,
                &branch_names
                    .iter()
                    .map(|b| b.as_str())
                    .collect::<Vec<&str>>(),
            )
        }

        Commands::Rename {
            current_name,
            new_name,
        } => {
            println!("[RENAME]");
            rename_sub_command(state, current_name, new_name)
        }
    }
}

// ---

/// The name of the hidden directory created within the target Git repository
/// to store jump-related metadata.
const JUMP_FOLDER: &str = ".jump";

/// The name of the JSON file where branch usage history and timestamps are saved.
const DATA_FILE: &str = "data.json";

fn ensure_jump_folder_exists(path: &PathBuf) -> Result<()> {
    let jump_store_dir = path.join(JUMP_FOLDER);
    let store_data_file = jump_store_dir.join(DATA_FILE);

    if !jump_store_dir.exists() {
        if let Err(e) = fs::create_dir(jump_store_dir) {
            return Err(anyhow!("Couldn't create {} dir: {}", JUMP_FOLDER, e));
        };

        let mut file = OpenOptions::new()
            .append(true)
            .open(path.join(".git").join("info").join("exclude"))
            .unwrap();

        if let Err(e) = writeln!(file, "\n{}", JUMP_FOLDER) {
            return Err(anyhow!("Couldn't write to file: {}", e));
        }
    }

    if !store_data_file.exists() {
        let mut file = File::create(store_data_file)?;
        if let Err(e) = file.write_all(b"{}") {
            return Err(anyhow!("Couldn't write to file: {}", e));
        }
    }

    Ok(())
}

struct InitData {
    main_worktree: PathBuf,
    active_worktree: PathBuf,
    branches: Vec<Branch>,
    worktrees: Vec<Worktree>,
}

// struct MainWorktree {
//     path: PathBuf,
// }

// impl MainWorktree {
//     pub fn git_dir(&self) -> &Path {
//         &self.path.join(".git")
//     }

//     pub fn jump_dir(&self) -> &Path {
//         &self.path.join(JUMP_FOLDER)
//     }
// }

fn init() -> Result<InitData> {
    let GitDirs {
        main_worktree,
        active_worktree,
    } = locate_git_repo_dirs()?;

    ensure_jump_folder_exists(&main_worktree)?;

    let raw_git_branches = read_raw_git_branches()?;

    let branch_names: Vec<&str> = raw_git_branches.iter().map(|s| s.as_str()).collect();

    let worktrees = list_worktrees()?;

    let branches = get_and_clean_branches(
        &main_worktree
            .parent()
            .unwrap() // for this to fail, main_worktree needs to be set to "/" or "" somehow
            .join(JUMP_FOLDER)
            .join(DATA_FILE),
        &branch_names,
    )?;

    Ok(InitData {
        main_worktree,
        active_worktree,
        branches,
        worktrees,
    })
}

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};

use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::PathBuf,
    process,
};

use git_jump::{
    app::InteractiveApp,
    command::{delete_sub_command, jump_to, list_sub_command, new_sub_command, rename_sub_command},
    git::{GitDirs, list_worktrees, locate_git_repo_dirs, read_raw_git_branches},
    list::{prep_available_branches, prep_available_worktrees},
    storage::get_and_clean_branches,
    types::{
        Branch, BranchDeleteResult, DATA_FILE, JUMP_FOLDER, MainWorktree, Model, ModifierKey,
        Worktree, get_active_worktree,
    },
    ui::{render_branch_deletion_res, render_branch_list, render_git_jump_error},
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
        current_name: Option<String>,
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
        Invocation::Interactive => {
            let w = get_active_worktree(&state.worktrees, &state.active_worktree);

            let branches = prep_available_branches(&state.branches, &state.worktrees);
            let worktrees = prep_available_worktrees(&state.worktrees, &state.active_worktree);

            let app = InteractiveApp::new(w.head.to_owned(), branches, worktrees);
            ratatui::run(|terminal| app.run(terminal))?;
        }

        Invocation::JumpTo(branch) => match jump_to(&state, branch, &[]) {
            Ok(info) => {
                println!("{}", info);
                process::exit(0)
            }
            Err(err) => {
                render_git_jump_error(err);
                process::exit(1)
            }
        },

        Invocation::Sub(cmd) => match cmd {
            Commands::List => {
                let active = get_active_worktree(&state.worktrees, &state.active_worktree);
                let branches = list_sub_command(&state);
                render_branch_list(&active.head, &branches, &state.worktrees);
                process::exit(0)
            }

            Commands::New { branch_name } => {
                match new_sub_command(&state, branch_name) {
                    Ok(info) => {
                        println!("{}", info);
                        process::exit(0)
                    }
                    Err(err) => {
                        render_git_jump_error(err);
                        process::exit(1)
                    }
                };
            }

            Commands::Delete { branch_names } => {
                match delete_sub_command(
                    &state,
                    &branch_names
                        .iter()
                        .map(|b| b.as_str())
                        .collect::<Vec<&str>>(),
                ) {
                    Ok(res) => {
                        render_branch_deletion_res(&res);
                        let exit_code = res
                            .iter()
                            .any(|r| matches!(r, BranchDeleteResult::Failed(..)))
                            as i32;
                        process::exit(exit_code)
                    }
                    Err(err) => {
                        render_git_jump_error(err);
                        process::exit(1)
                    }
                }
            }

            Commands::Rename {
                current_name,
                new_name,
            } => match rename_sub_command(&state, current_name.as_deref(), new_name) {
                Ok(info) => {
                    println!("{}", info);
                    process::exit(0)
                }
                Err(err) => {
                    render_git_jump_error(err);
                    process::exit(1)
                }
            },
        },
    }

    Ok(())
}

// ---

fn ensure_jump_folder_exists(main_worktree: &MainWorktree) -> Result<()> {
    let jump_store_dir = main_worktree.jump_dir();
    if !jump_store_dir.exists() {
        if let Err(e) = fs::create_dir(jump_store_dir) {
            return Err(anyhow!("Couldn't create {} dir: {}", JUMP_FOLDER, e));
        };

        let mut file = OpenOptions::new()
            .append(true)
            .open(main_worktree.git_dir().join("info").join("exclude"))
            .unwrap();

        if let Err(e) = writeln!(file, "\n{}", JUMP_FOLDER) {
            return Err(anyhow!("Couldn't write to file: {}", e));
        }
    }

    let store_data_file = main_worktree.data_file();
    if !store_data_file.exists() {
        let mut file = File::create(store_data_file)?;
        if let Err(e) = file.write_all(b"{}") {
            return Err(anyhow!("Couldn't write to file: {}", e));
        }
    }

    Ok(())
}

struct InitData {
    main_worktree: MainWorktree,
    active_worktree: PathBuf,
    branches: Vec<Branch>,
    worktrees: Vec<Worktree>,
}

fn init() -> Result<InitData> {
    let GitDirs {
        main_worktree,
        active_worktree,
    } = locate_git_repo_dirs()?;

    ensure_jump_folder_exists(&main_worktree)?;

    let raw_git_branches = read_raw_git_branches()?;

    let branch_names: Vec<&str> = raw_git_branches.iter().map(|s| s.as_str()).collect();

    let worktrees = list_worktrees()?;

    let branches = get_and_clean_branches(&main_worktree.data_file(), &branch_names)?;

    Ok(InitData {
        main_worktree,
        active_worktree,
        branches,
        worktrees,
    })
}

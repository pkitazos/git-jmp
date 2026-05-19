use anyhow::{Context, Result, anyhow};
use regex::Regex;

use std::collections::HashMap;

use std::path::PathBuf;
use std::{
    fs::{self, File},
    io::Write,
};

use std::sync::LazyLock;

use crate::git::{RawWorktree, locate_git_repo_dirs, read_raw_git_branches, read_raw_worktrees};
use crate::storage::{JUMP_FOLDER, MainWorktree, clean_and_save_jump_data, load_jump_data};
use crate::types::{Branch, Head, Worktree};

pub struct Model {
    pub main_worktree: MainWorktree,
    pub active_worktree: PathBuf,

    pub branches: Vec<Branch>,
    pub worktrees: Vec<Worktree>,
}

impl Model {
    pub fn init() -> Result<Self> {
        let dirs = locate_git_repo_dirs()?;

        ensure_jump_folder_exists(&dirs.main_worktree)?;

        let branch_names = read_raw_git_branches()?;
        let raw_worktrees = read_raw_worktrees()?;

        let mut jump_data = load_jump_data(&dirs.main_worktree.data_file())?;
        clean_and_save_jump_data(
            &dirs.main_worktree.data_file(),
            &mut jump_data,
            &branch_names,
        )?;

        let branches = construct_branches(&branch_names, &jump_data);
        let worktrees = construct_worktrees(raw_worktrees, &jump_data);

        Ok(Self {
            main_worktree: dirs.main_worktree,
            active_worktree: dirs.active_worktree,
            branches,
            worktrees,
        })
    }
}

// ---

// so the reason these can't just be constant values is that initialising a Regex
// only happens at runtime, because for potentially very large patterns constructing the NFA/DF
// may actually require heap allocations and a `const` needs to be compile-time computable
// so using LazyLock means its computed the first time we need it,
// but every other time we need it it's using the same computed pattern
fn semver_exact_pattern(haystack: &str) -> bool {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+\.\d+\.\d+$").unwrap());
    RE.is_match(haystack)
}

// todo: figure out where this should be called
fn fetch_latest_version() -> Result<String> {
    let response: serde_json::Value =
        ureq::get("https://api.github.com/repos/pkitazos/git-jump/releases/latest")
            .header("User-Agent", "git-jump")
            .call()
            .context("failed to fetch latest release from GitHub")?
            .body_mut()
            .read_json()
            .context("failed to parse GitHub response")?;

    let tag = response["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow!("no tag_name in GitHub response"))?;

    let version = tag.strip_prefix('v').unwrap_or(tag);

    if semver_exact_pattern(version) {
        Ok(version.to_owned())
    } else {
        Err(anyhow!("tag '{}' is not a valid semver version", tag))
    }
}

fn ensure_jump_folder_exists(main_worktree: &MainWorktree) -> Result<()> {
    let jump_store_dir = main_worktree.jump_dir();
    if !jump_store_dir.exists() {
        if let Err(e) = fs::create_dir(jump_store_dir) {
            return Err(anyhow!("Couldn't create {} dir: {}", JUMP_FOLDER, e));
        };

        let mut file = fs::OpenOptions::new()
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

fn construct_branches(names: &[String], jump_data: &HashMap<String, u64>) -> Vec<Branch> {
    names
        .iter()
        .map(|b| Branch {
            name: b.to_string(),
            last_switch: jump_data.get(b).copied().unwrap_or(0u64),
        })
        .collect()
}

fn construct_worktrees(raw: Vec<RawWorktree>, jump_data: &HashMap<String, u64>) -> Vec<Worktree> {
    raw.into_iter()
        .map(|w| Worktree {
            dir: w.dir,
            head: match w.branch {
                Some(name) => Head::Branch(Branch {
                    last_switch: jump_data.get(&name).copied().unwrap_or(0u64),
                    name,
                }),
                None => Head::Detached { sha: w.sha },
            },
        })
        .collect()
}

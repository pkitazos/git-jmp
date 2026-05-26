use anyhow::{Context, Result, anyhow};
use std::{collections::HashMap, path::PathBuf, process::Command};

use crate::storage::MainWorktree;

pub struct GitDirs {
    pub main_worktree: MainWorktree,
    pub active_worktree: PathBuf,
}

pub fn locate_git_repo_dirs() -> Result<GitDirs> {
    let out = git_command("rev-parse", &["--path-format=absolute", "--git-common-dir"])?;
    let main_worktree = MainWorktree {
        project_root_dir: PathBuf::from(out.stdout).parent().unwrap().to_path_buf(),
    };

    let out = git_command("rev-parse", &["--show-toplevel"])?;
    let active_worktree = PathBuf::from(out.stdout);

    Ok(GitDirs {
        main_worktree,
        active_worktree,
    })
}

pub fn read_raw_git_branches() -> Result<Vec<String>> {
    let out = git_command("branch", &["--format=%(refname:short)"])?;

    let branches: Vec<String> = out
        .stdout
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect();

    Ok(branches)
}

pub struct RawWorktree {
    pub dir: PathBuf,
    pub branch: Option<String>,
    pub sha: String,
}

pub fn read_raw_worktrees() -> Result<Vec<RawWorktree>> {
    let out = git_command("worktree", &["list", "--porcelain"])?;

    out.stdout
        .split("\n\n")
        .filter(|s| !s.is_empty())
        .map(|r| {
            let entry_lines: Vec<&str> = r.lines().collect();
            parse_worktree_entry(&entry_lines)
        })
        .collect::<Result<Vec<RawWorktree>>>()
}

pub fn fetch_remotes() -> Result<Vec<String>> {
    let out = git_command("remote", &[])?;
    Ok(out.stdout.lines().map(|r| r.trim().to_string()).collect())
}

pub struct RemoteBranch {
    pub remote: String,
    pub name: String,
}

pub fn fetch_remote_branches(remote: &str) -> Result<Vec<RemoteBranch>> {
    let out = git_command("ls-remote", &["--heads", remote])?;

    let branches: Vec<RemoteBranch> = out
        .stdout
        .lines()
        .filter_map(|line| line.split('\t').nth(1))
        .map(|r| RemoteBranch {
            remote: remote.to_string(),
            name: r.trim_start_matches("refs/heads/").to_string(),
        })
        .collect();

    Ok(branches)
}

pub fn read_cached_remote_branches() -> Result<HashMap<String, Vec<String>>> {
    let out = git_command(
        "for-each-ref",
        &["--format=%(refname:strip=2)", "refs/remotes/"],
    )?;

    let mut remote_branches: HashMap<String, Vec<String>> = HashMap::new();
    for (r, b) in out
        .stdout
        .lines()
        .filter(|line| !line.ends_with("/HEAD"))
        .filter_map(|line| line.split_once("/"))
    {
        remote_branches
            .entry(r.to_string())
            .or_default()
            .push(b.to_string());
    }

    Ok(remote_branches)
}

fn parse_worktree_entry(lines: &[&str]) -> Result<RawWorktree> {
    let mut dir: Option<PathBuf> = None;
    let mut sha: Option<&str> = None;
    let mut branch: Option<&str> = None;
    let mut detached = false;
    let mut bare = false;

    for line in lines.iter() {
        match line.split_once(' ') {
            Some((key, val)) => match key {
                "worktree" => dir = Some(PathBuf::from(val)),
                "HEAD" => sha = val.get(..7).or(Some(val)),
                "branch" => {
                    branch = match val.strip_prefix("refs/heads/") {
                        Some(name) => Some(name),
                        None => Some(val),
                    }
                }
                _ => {}
            },
            None => match *line {
                "bare" => bare = true,
                "detached" => detached = true,
                _ => {}
            },
        }
    }

    match (bare, detached, sha, branch, dir) {
        (true, _, _, _, _) => Err(anyhow!("Bare repo not supported")),
        (false, true, Some(sha), None, Some(dir)) => Ok(RawWorktree {
            dir,
            branch: None,
            sha: sha.to_string(),
        }),
        (false, false, Some(sha), Some(name), Some(dir)) => Ok(RawWorktree {
            dir,
            sha: sha.to_string(),
            branch: Some(name.to_string()),
        }),
        _ => Err(anyhow!("Malformed worktree record: {:?}", lines)),
    }
}

pub struct GitOutput {
    pub stdout: String,
    pub stderr: String,
}

pub fn git_command(sub_cmd: &str, args: &[&str]) -> Result<GitOutput> {
    let mut cmd = Command::new("git");
    cmd.arg(sub_cmd);
    cmd.args(args);

    let output = cmd.output()?;

    if !output.status.success() {
        return Err(anyhow!("{}", String::from_utf8_lossy(&output.stderr)));
    };

    Ok(GitOutput {
        stdout: String::from_utf8(output.stdout)
            .context("git returned non-UTF-8 bytes on stdout")?
            .trim()
            .to_owned(),
        stderr: String::from_utf8(output.stderr)
            .context("git returned non-UTF-8 bytes on stderr")?
            .trim()
            .to_owned(),
    })
}

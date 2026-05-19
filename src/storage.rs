use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

pub struct MainWorktree {
    pub project_root_dir: PathBuf,
}

/// The name of the hidden directory created within the target Git repository
/// to store jump-related metadata.
pub const JUMP_FOLDER: &str = ".jump";

/// The name of the JSON file where branch usage history and timestamps are saved.
pub const DATA_FILE: &str = "data.json";

impl MainWorktree {
    pub fn root(&self) -> &Path {
        &self.project_root_dir
    }

    pub fn git_dir(&self) -> PathBuf {
        self.project_root_dir.join(".git")
    }

    pub fn jump_dir(&self) -> PathBuf {
        self.root().join(JUMP_FOLDER)
    }

    pub fn data_file(&self) -> PathBuf {
        self.jump_dir().join(DATA_FILE)
    }
}

// ---

type BranchCollection = HashMap<String, u64>;

/// Reads the historical data from disk, updates the timestamp in memory,
/// and immediately flushes the changes back to `.jump/data.json`.
pub fn update_branch_last_switch(data_file: &Path, name: &str, last_switch: u64) -> Result<()> {
    let mut jump_data: BranchCollection = load_jump_data(data_file)?;

    set_branch_timestamp(&mut jump_data, name, last_switch);

    save_branches_jump_data(data_file, &jump_data)
}

/// Reads the current data from disk, applies the pure rename transformation,
/// and safely writes the updated history back to `.jump/data.json`.
pub fn rename_jump_data_branch(data_file: &Path, current_name: &str, new_name: &str) -> Result<()> {
    let mut jump_data: BranchCollection = load_jump_data(data_file)?;

    rename_branch(&mut jump_data, current_name, new_name);

    save_branches_jump_data(data_file, &jump_data)
}

/// Reads the current data from disk, filters out the specified branches,
/// and writes the clean data back to `.jump/data.json`.
pub fn delete_jump_data_branch(data_file: &Path, branch_names: &[&str]) -> Result<()> {
    let mut jump_data: BranchCollection = load_jump_data(data_file)?;

    delete_branches(&mut jump_data, branch_names);

    save_branches_jump_data(data_file, &jump_data)
}

/// Loads jump data from disk, normalising the on-disk format if needed.
///
/// Supports both:
/// - the legacy V1 format (`{ "branch": { "name": <string>, "lastSwitch": <timestamp> } }`)
/// - the current V2 format (`{ "branch": <timestamp> }`)
///
/// When a V1 file is encountered, the original is copied to `<data_file>.v1.bak`
/// before the caller writes it back in V2 form.
pub fn load_jump_data(data_file: &Path) -> Result<HashMap<String, u64>> {
    let file = File::open(data_file)?;
    let reader = BufReader::new(file);

    let branches: OnDisk = serde_json::from_reader(reader)
        .with_context(|| format!("failed to deserialize jump data at {}", data_file.display()))?;

    match branches {
        OnDisk::V2(_) => {}
        OnDisk::V1(_) => {
            // This is the side-effect
            create_backup(data_file)?;
        }
    };

    Ok(parse_from_disk(branches))
}

pub fn clean_and_save_jump_data(
    data_file: &Path,
    jump_data: &mut HashMap<String, u64>,
    valid_branch_names: &[String],
) -> Result<()> {
    keep_branches(
        jump_data,
        &valid_branch_names
            .iter()
            .map(|b| b.as_str())
            .collect::<Vec<_>>(),
    );
    save_branches_jump_data(data_file, jump_data)?;
    Ok(())
}

// --- actual file I/O

#[derive(Deserialize)]
#[serde(untagged)]
enum OnDisk {
    V2(HashMap<String, u64>),
    V1(HashMap<String, BranchEntry>),
}

#[derive(Deserialize)]
struct BranchEntry {
    #[serde(rename = "lastSwitch")]
    last_switch: u64,
}

fn parse_from_disk(data: OnDisk) -> BranchCollection {
    match data {
        OnDisk::V2(hash_map) => hash_map,
        OnDisk::V1(hash_map) => hash_map
            .into_iter()
            .map(|(k, v)| (k, v.last_switch))
            .collect::<BranchCollection>(),
    }
}

fn create_backup(data_file: &Path) -> Result<()> {
    let backup_file = data_file.with_extension("json.v1.bak");
    fs::copy(data_file, backup_file).with_context(|| {
        format!(
            "failed to back up old jump data file for {}",
            data_file.display()
        )
    })?;

    Ok(())
}

fn save_branches_jump_data(data_file: &Path, jump_data: &BranchCollection) -> Result<()> {
    let file = File::create(data_file)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &jump_data).with_context(|| {
        format!(
            "failed to write serialized branch data to {}",
            data_file.display()
        )
    })?;

    Ok(())
}

// pure utils

fn set_branch_timestamp(jump_data: &mut BranchCollection, name: &str, last_switch: u64) {
    jump_data.insert(name.to_owned(), last_switch);
}

fn rename_branch(jump_data: &mut BranchCollection, current_name: &str, new_name: &str) {
    if let Some(last_switch) = jump_data.remove(current_name) {
        jump_data.insert(new_name.to_owned(), last_switch);
    };
}

enum FilterMode {
    Keep,
    Delete,
}

fn filter_jump_data(jump_data: &mut BranchCollection, branch_names: &[&str], mode: FilterMode) {
    let set: HashSet<&str> = branch_names.iter().copied().collect();
    jump_data.retain(|k, _| {
        let in_set = set.contains(k.as_str());
        match mode {
            FilterMode::Keep => in_set,
            FilterMode::Delete => !in_set,
        }
    });
}

fn keep_branches(jump_data: &mut BranchCollection, branch_names: &[&str]) {
    filter_jump_data(jump_data, branch_names, FilterMode::Keep);
}

fn delete_branches(jump_data: &mut BranchCollection, branch_names: &[&str]) {
    filter_jump_data(jump_data, branch_names, FilterMode::Delete);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_data() -> BranchCollection {
        HashMap::from([
            ("main".to_string(), 100),
            ("dev".to_string(), 50),
            ("feature".to_string(), 10),
        ])
    }

    // set_branch_timestamp

    #[test]
    fn set_timestamp_inserts_new_branch() {
        let mut data = HashMap::new();
        set_branch_timestamp(&mut data, "main", 42);
        assert_eq!(data.get("main"), Some(&42));
    }

    #[test]
    fn set_timestamp_updates_existing_branch() {
        let mut data = sample_data();
        set_branch_timestamp(&mut data, "main", 999);
        assert_eq!(data.get("main"), Some(&999));
    }

    // rename_branch

    #[test]
    fn rename_moves_timestamp_to_new_key() {
        let mut data = sample_data();
        rename_branch(&mut data, "main", "primary");
        assert_eq!(data.get("primary"), Some(&100));
        assert!(!data.contains_key("main"));
    }

    #[test]
    fn rename_nonexistent_is_noop() {
        let mut data = sample_data();
        let before = data.clone();
        rename_branch(&mut data, "nonexistent", "new");
        assert_eq!(data, before);
    }

    // delete_branches

    #[test]
    fn delete_removes_specified_branches() {
        let mut data = sample_data();
        delete_branches(&mut data, &["main", "dev"]);
        assert!(!data.contains_key("main"));
        assert!(!data.contains_key("dev"));
        assert!(data.contains_key("feature"));
    }

    #[test]
    fn delete_nonexistent_is_noop() {
        let mut data = sample_data();
        let before = data.clone();
        delete_branches(&mut data, &["nonexistent"]);
        assert_eq!(data.len(), 3);
        assert_eq!(data, before);
    }

    #[test]
    fn delete_empty_list_keeps_all() {
        let mut data = sample_data();
        delete_branches(&mut data, &[]);
        assert_eq!(data.len(), 3);
    }

    // keep_branches

    #[test]
    fn keep_retains_only_specified() {
        let mut data = sample_data();
        keep_branches(&mut data, &["main"]);
        assert_eq!(data.len(), 1);
        assert_eq!(data.get("main"), Some(&100));
    }

    #[test]
    fn keep_empty_list_removes_all() {
        let mut data = sample_data();
        keep_branches(&mut data, &[]);
        assert!(data.is_empty());
    }

    // parse_from_disk

    #[test]
    fn parse_v2_format() {
        let json = r#"{"main": 100, "dev": 50}"#;
        let on_disk: OnDisk = serde_json::from_str(json).unwrap();
        let result = parse_from_disk(on_disk);
        assert_eq!(result.get("main"), Some(&100));
        assert_eq!(result.get("dev"), Some(&50));
    }

    #[test]
    fn parse_v1_format() {
        let json = r#"{"main": {"lastSwitch": 100}, "dev": {"lastSwitch": 50}}"#;
        let on_disk: OnDisk = serde_json::from_str(json).unwrap();
        let result = parse_from_disk(on_disk);
        assert_eq!(result.get("main"), Some(&100));
        assert_eq!(result.get("dev"), Some(&50));
    }
}

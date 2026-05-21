use anyhow::{Context, Result, anyhow};
use crossterm::style::Stylize;
use regex::Regex;

pub fn check_pkg_version() {
    let Ok(latest_version) = fetch_latest_version() else {
        return;
    };

    let version = env!("CARGO_PKG_VERSION");

    if version != latest_version {
        eprintln!(
            "\n{} {} → {}",
            "Update available:".bold(),
            version.dark_grey(),
            latest_version.green(),
        );
    }
}

fn fetch_latest_version() -> Result<String> {
    let response: serde_json::Value =
        ureq::get("https://api.github.com/repos/pkitazos/git-jmp/releases/latest")
            .header("User-Agent", "git-jmp")
            .call()
            .context("failed to fetch latest release from GitHub")?
            .body_mut()
            .read_json()
            .context("failed to parse GitHub response")?;

    let tag = response["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow!("no tag_name in GitHub response"))?;

    let version = tag.strip_prefix('v').unwrap_or(tag);

    // safe to unwrap cause it's a valid regex
    if Regex::new(r"^\d+\.\d+\.\d+$").unwrap().is_match(version) {
        Ok(version.to_owned())
    } else {
        Err(anyhow!("tag '{}' is not a valid semver version", tag))
    }
}

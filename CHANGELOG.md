# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0]

### Changed
- Rewritten in Rust; ships as a single native binary with no runtime dependencies. Binary renamed from `git-jump` to `git-jmp` (invoked as `git jmp`).
- Sub-commands renamed to shorter UNIX-style names:

| **TS**             | **Rust**           |
| ------------------ | ------------------ |
| `--list` / `-l`    | `ls`               |
| `delete`           | `rm`               |
| `rename`           | `mv`               |
| `new`              | `new` (unchanged)  |
| `-v` / `--version` | `-V` / `--version` |

- Branches checked out in other worktrees are now selectable in interactive mode. Selecting one shows the path and a `cd` command instead of switching.
- `git jmp mv <new-name>` now defaults to renaming the current branch when only one argument is given (previously required both old and new names).
- Interactive mode renders in an alternate screen buffer.
- Jump data format simplified from `{"branch": {"name": "...", "lastSwitch": N}}` to `{"branch": N}`. Existing data files are auto-migrated with a `.v1.bak` backup.
- `git jmp ls` now shows `*` / `+` markers and colours when printing to a terminal (plain text when piped, same as before).
- Highlight colour in interactive mode changed from blue to magenta.
- Update check now queries GitHub Releases instead of the npm registry, and runs after any successful command rather than only in interactive mode.
- HEAD is now displayed as a 7-character short SHA instead of the full hash.
- Reorganised project layout: shell integration scripts moved to `shell/`, images moved to `docs/img/`.

### Added
- Shell integration via `git jmp init <shell>`. Prints a `jmp` wrapper function for your shell (bash, zsh, or fish). With the wrapper sourced, selecting a branch checked out in another worktree automatically `cd`s into that worktree instead of just printing the path.
- Configuration system with global (`~/.config/git-jmp/config.toml`) and per-repo (`.jump/config.toml`) config files. Local config overrides global field-by-field.
- Vim mode (`--vim-mode` flag or `vim_mode` config option). Adds Normal/Input mode split with `j`/`k` navigation and `q` to quit.
- Remote branch support in interactive mode via `-r`/`--include-remotes` flag or the `sources` config option. Remotes can also be listed with `git jmp ls -r`.
- Force-delete flag: `git jmp rm -f` / `--force` for branches not fully merged.
- Quick-select hint style is configurable (`quick_select_hint`: `"full"`, `"compact"`, or `"hidden"`).
- Auto-update checking can be disabled via the `auto_check_updates` config option.

### Removed
- Pass-through of extra arguments to `git switch`. Multi-argument invocations like `git jump my-branch --discard-changes` are no longer supported; only a single branch argument is accepted. I'm working on a clean way to handle the pass-through in a later update.

## [0.1.5]

### Fixed
- Windows: `git jump` no longer recurses past the filesystem root when run outside a git repo (the `folder === "/"` termination check never matched on Windows roots like `C:\`).
- Windows: worktree path lookups no longer fail because of separator/case mismatches between git's forward-slash output and `process.cwd()`'s native paths.

## [0.1.4] - 2026-04-21

### Added
- Worktree support. `git jump` now works from inside linked worktrees, which previously failed with `"You're not in Git repo. There is no Git repository in current or any parent folder."`
- Branches checked out in other worktrees are listed at the bottom with their worktree path, they are shown for visibility but are not selectable.

## [0.1.3] - 2026-04-17

### Fixed
- `git jump <branch>` now preserves the native `git switch` error when the switch fails for reasons unrelated to the branch name (e.g. uncommitted changes blocking a switch to a remote-only branch), instead of masking it with "does not match any branch".
- Fuzzy search now only runs when a single argument is provided. Multi-argument invocations (e.g. `git jump my-branch --discard-changes`) pass through to `git switch` directly without dropping flags or silently fuzzy-matching on the first argument.
- `git jump <current-branch>` now shows a concise "Staying on \<branch\>" message instead of git's raw tracking/status output, matching the interactive-mode behavior.

### Changed
- Branch name in the "No Match" error is no longer coloured.
- Terminal setup instructions expanded to cover VS Code's integrated terminal.

## [0.1.2] - 2026-04-14

### Added
- Terminal setup instructions for Ghostty and Zed's integrated terminal.

### Changed
- Commented out Homebrew installation references pending tap setup.

## [0.1.1] - 2026-04-14

### Added
- <kbd>Cmd</kbd>+<kbd>←</kbd> / <kbd>Cmd</kbd>+<kbd>→</kbd> jump to the start/end of the search string.
- <kbd>Opt</kbd>+<kbd>←</kbd> / <kbd>Opt</kbd>+<kbd>→</kbd> navigate the search string word-by-word.
- <kbd>Opt</kbd>+<kbd>Backspace</kbd> deletes the word before the cursor.
- Additional input-clearing key combos.

## [0.1.0] - 2026-04-14

### Added
- Initialised a standard `CHANGELOG.md` for the modernised fork.
- Specified `pnpm` as the strict package manager via the `engines` and `packageManager` fields.

### Changed
- Forked the project from the original `git-jump` (v0.3.1) by Mykola Harmash to continue active maintenance.
- Renamed the package to the scoped namespace `@pkitazos/git-jump`.
- Updated documentation (`README.md`) to reflect new ownership, Homebrew installation paths, and macOS setup instructions.
- Switched the build scripts to use `pnpm` instead of `npm`.

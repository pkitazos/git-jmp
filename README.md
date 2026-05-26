![git-jump CLI logo](https://raw.githubusercontent.com/pkitazos/git-jump/main/docs/img/banner.png)

# Git-Jmp

A fast, interactive branch switcher for Git with fuzzy search, recency sorting, and worktree support.

- **Interactive UI** to view and switch between branches and worktrees
- **Recency sorting:** your most recently used branches float to the top
- **Fuzzy jump:** `git jmp 481` → `git switch feat/issue-481-auth-refactor`
- **Quick select:** jump to any of your top 10 branches with a single keystroke
- **Vim mode** optional `Normal`/`Input` mode split + `j`/`k` navigation for the interactive list

<!-- <p align="center">
  <img src="https://raw.githubusercontent.com/pkitazos/git-jump/main/img/demo.gif" alt="git jmp interactive interface" width="600px" style="border-radius: 5px;" />
</p> -->

## Install

```shell
cargo install git-jmp
```

<!--or using Homebrew

```shell
brew install git-jmp
```-->

## Usage

### Interactive Mode

Run without arguments to launch the interactive UI:

```shell
git jmp
```

- When you first start using `git jmp` branches are sorted alphabetically, but as you switch around your jump history is tracked and the list is sorted with the most recently jumped-to branches near the top.
- Navigate with arrow keys or, if vim mode is enabled, with `j`/`k`. Hit enter to switch to the selected branch.
- Start typing to filter the list with a fuzzy search. You don't have to be precise, just type enough to narrow it down.
- Quick-jump to any of your top 10 most recently visited branches using <kbd>Option</kbd>+<kbd>\<number\></kbd> (or <kbd>Alt</kbd>+<kbd>\<number\></kbd> on Linux). You may need to configure your terminal for this to work, see [terminal configuration](docs/terminal-config.md).

### Direct Jump

Jump to a branch without opening the interactive UI. You can use the exact name or a partial match. `git-jmp` checks for an exact match first, then falls back to the best fuzzy match.

```shell
git jmp <branch name or fuzzy match>
```

If you use feature branches with unique issue numbers, this makes switching between them very quick:

```shell
git jmp 481     # switches to feat/issue-481-auth-refactor
git jmp signup  # switches to feat/user-signup-flow
```

### Subcommands

#### new

```shell
git jmp new <branch name>
```

Creates and switches to a new branch (`git switch --create` under the hood) and records it as the most recently visited branch.

#### mv

```shell
git jmp mv [<old name>] <new name>
```

Renames a branch (`git branch --move` under the hood) and updates the jump data. Omit `<old name>` to rename the current branch.

#### rm

```shell
git jmp rm [-f] <branch name> [<branch name>, ...]
```

Deletes one or more branches (`git branch -d` under the hood) and removes them from the jump data. Pass `-f` or `--force` for branches that haven't been fully merged (equivalent to `git branch -D`).

#### ls

```shell
git jmp ls
```

Lists all local branches with the same markers as `git branch` (`*` for current, `+` for worktree branches). The only difference is that when piped, the output is plain branch names with no prefixes (unlike `git branch`), just one branch per line, ready for scripting!

Pass `-r` or `--include-remotes` to also list remote branches. This queries your remotes directly rather than relying on your local cache (unlike `git branch -r`).

## Configuration

You can configure `git-jmp` globally, per-project, or with CLI flags. Configuration is written in TOML:

```toml
[general]
auto_check_updates = true
vim_mode = false

[appearance]
quick_select_hint = "full" # or "compact" or "hidden"
```

- **`auto_check_updates`** - whether `git-jmp` checks for new versions on startup. Disable this if you'd rather skip the network trip.
- **`vim_mode`** - splits the interactive list into `Normal` mode (navigate) and `Input` mode (type to search) and allows you to move up and down the list using `j`/`k`.
- **`quick_select_hint`** - controls the hint shown on the search input line: `"full"` is the verbose default, `"compact"` shows just the modifier key and number, `"hidden"` shows nothing.

Global config location depends on your OS:

| OS      | Path                                                                      |
| ------- | ------------------------------------------------------------------------- |
| Linux   | `~/.config/git-jmp/config.toml` (or `$XDG_CONFIG_HOME/git-jmp/config.toml`) |
| macOS   | `~/Library/Application Support/git-jmp/config.toml`                       |
| Windows | `%APPDATA%\git-jmp\config.toml`                                           |

Local config goes in `.jump/config.toml` at the root of your repository. Local values override global ones field by field, so you don't need to specify every field every time.

## Migrating from `git-jump`

If you previously used [mykolaharmash/git-jump](https://github.com/mykolaharmash/git-jump) or the [@pkitazos/git-jump](https://www.npmjs.com/package/@pkitazos/git-jump) ts fork, you can install `git-jmp` and drop it into any existing project and your jump data will be carried over automatically.

The subcommand names have changed: `rename` → `mv`, `delete` → `rm`, `--list` → `ls`.

### Differences from the original

`git-jmp` is a ground-up Rust rewrite. Beyond the port itself, it includes:

- Worktree-aware interactive UI and branch listing
- Per-project and global TOML configuration
- Vim mode for the interactive list
- Configurable quick-select hints
- Remote branch listing via `ls -r`
- Full cursor navigation in the search input (word jump, Home/End, kill line)

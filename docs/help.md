### `git jmp --help` 

```
A fast, interactive branch switcher for Git with fuzzy search, recency
sorting, and worktree support.

Run with no arguments to launch the interactive UI. Branches are sorted by
recency, the ones you switch to most often float to the top. Start typing
to fuzzy-filter the list.

Jump directly to a branch without the UI by passing a name or partial
match:

  git jmp 481       switches to feat/issue-481-auth-refactor
  git jmp signup    switches to feat/user-signup-flow
  git jmp -         switches to the previously checked-out branch

An exact match is tried first, then the best fuzzy match.

Usage: git jmp [BRANCH]
       git jmp <COMMAND>

Commands:
  ls    List all branches
  new   Create a new branch and switch to it
  rm    Delete one or more branches
  mv    Rename a branch
  help  Print this message or the help of the given subcommand(s)

Arguments:
  [BRANCH]
          Jump to a branch by exact or fuzzy name match, or `-` for the
          previously checked-out branch.

          Checks for an exact match first, then falls back to the best
          fuzzy match. You can use just part of the name, e.g. `git jmp
          481` to match `feat/issue-481-auth-refactor`.

Interactive mode options:
  These flags only apply when launching the interactive UI (i.e. when
  no BRANCH or subcommand is given). They are ignored otherwise.

      --vim-mode
          Enable vim-style navigation (j/k, Normal/Input mode split)

  -r, --include-remotes
          Include any branches which exist on any remote in the list

Options:
  -h, --help     Print help (see a summary with '-h')
  -V, --version  Print version

Interactive mode keybindings:
  General:
    Enter           Switch to the selected branch
    Ctrl+C          Cancel and exit
    Alt+0..9        Quick-select a branch by its position
                    (⌥+0..9 on macOS)

  Navigation:
    ↑ / ↓           Move up/down the list
    j / k           Move up/down (vim mode only)

  Search input (default mode, or Input mode in vim):
    Type            Fuzzy-filter branches by name
    ← / →           Move cursor left/right by one character
    Alt+← / Alt+→   Move cursor by one word
    Home / Ctrl+A   Jump to start of input
    End  / Ctrl+E   Jump to end of input
    Backspace       Delete character before cursor
    Delete          Delete character at cursor
    Alt+Backspace   Delete previous word
    Ctrl+U          Delete from cursor to start of input
    Ctrl+K          Delete from cursor to end of input
    Ctrl+W          Clear the entire input

  Vim mode only:
    i               Enter Input mode (to type a search)
    Esc             Return to Normal mode
    q               Cancel and exit (Normal mode only)

Configuration:
  Global config: ~/.config/git-jmp/config.toml
  Local config:  .jump/config.toml (at repo root, overrides global)

  See https://github.com/pkitazos/git-jmp#configuration for all options.

Support:
  https://github.com/pkitazos/git-jmp
```

### `git jmp -h` 

```
A fast, interactive branch switcher for Git.

Usage: git jmp [BRANCH]
       git jmp <COMMAND>

Commands:
  ls    List all branches
  new   Create a new branch and switch to it
  rm    Delete one or more branches
  mv    Rename a branch
  help  Print this message or the help of the given subcommand(s)

Arguments:
  [BRANCH]  Branch to jump to (name, fuzzy match, or `-` for previous)

Interactive mode options:
      --vim-mode         Vim navigation (interactive mode only)
  -r, --include-remotes  Include remote branches (interactive mode only)

Options:
  -h, --help     Print help (see more with '--help')
  -V, --version  Print version
```

### `git jmp ls --help`

```
List all branches

Shows local branches with the same markers as `git branch` (* for current,
+ for worktree branches). When piped, outputs plain branch names with no
prefixes, one per line, ready for scripting.

Usage: git jmp ls [OPTIONS]

Options:
  -r, --include-remotes
          Also list remote branches (queries remotes directly, not the
          local cache)

  -h, --help
          Print help (see a summary with '-h')
```

### `git jmp new --help`

```
Create a new branch and switch to it

Runs `git switch --create` under the hood and records the new branch as
the most recently visited in your jump data.

Usage: git jmp new <BRANCH_NAME>

Arguments:
  <BRANCH_NAME>  Name for the new branch

Options:
  -h, --help  Print help (see a summary with '-h')
```

### `git jmp rm --help`

```
Delete one or more branches

Runs `git branch -d` under the hood and removes deleted branches from
your jump data. Use -f for branches that haven't been fully merged
(equivalent to `git branch -D`).

Usage: git jmp rm [OPTIONS] <BRANCH_NAMES>...

Arguments:
  <BRANCH_NAMES>...  Branches to delete

Options:
  -f, --force  Force-delete branches not yet fully merged
  -h, --help   Print help
```

### `git jmp mv --help` 

```
Rename a branch

Runs `git branch --move` under the hood and updates your jump data.

Usage: git jmp mv <NEW_NAME>
       git jmp mv <CURRENT_NAME> <NEW_NAME>

Arguments:
  <CURRENT_NAME>  Branch to rename (defaults to the current branch when
                  only one argument is given)
  <NEW_NAME>      New name for the branch

Options:
  -h, --help  Print help
```

# Todo

- [x] display errors nicely
  - [ ] `rm` should not lose native git command info in its output `Deleted branch <branch name> (was <sha>).`
  - [ ] `rm` should remove newline char between multiple failure messages
  - [ ] `new` should also not lose native git command info `Switched to a new branch '<branch name>'`
  - [x] `mv` is broken
- [ ] `git jump -` should work like `git switch -` it should not fuzzy match on some branch
- [x] should be able to call `--version / -v` and `--help / -h` from anywhere on the system (comes by default with clap)
- [ ] finish wiring up interactive mode
  - [ ] when idle, branches should still be ranked, except head is pinned
  - [ ] in interactive mode you should be able to actually press `Enter` to jump to a branch (literally the whole point)
  - [ ] if you're hovering a worktree entry, `Enter` should cd you into that directory (maybe we add a little status message about this)
  - [ ] if you're in normal mode and switch to interactive mode the highlighted line should reset (or maybe just hide?)
  - [ ] actually fix the styles to not use the blocky look
  - [ ] disable mouse scroll
- [x] query github for latest semver, not npm
- [ ] configurable interactive prompt (option to hide the quick select helper text, set vim mode)

## Configuration (potentially)

```toml
[general]
vim_mode = false
default_pool_sources = "local_only" # "include_remotes" (maybe even configurable as to which remotes to include)

[appearrance]
quick_select_helper = "full" # or "numbers_only" or "off"
theme = "default" # theme support comes in later
```

## Before release

- [ ] update license
- [ ] update readme
- [ ] update name
- [ ] update help messages
- [ ] open issue upstream
- [ ] unlink from fork network

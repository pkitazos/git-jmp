# Todo

- [ ] finish wiring up interactive mode
  - [ ] when idle, branches should still be ranked, except head is pinned
  - [ ] in interactive mode you should be able to actually press `Enter` to jump to a branch (literally the whole point)
  - [ ] if you're hovering a worktree entry, `Enter` should cd you into that directory (maybe we add a little status message about this)
  - [ ] if you're in normal mode and switch to interactive mode the highlighted line should reset (or maybe just hide?)
  - [ ] actually fix the styles to not use the blocky look
  - [ ] disable mouse scroll
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

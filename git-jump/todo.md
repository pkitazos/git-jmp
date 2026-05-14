# Todo
- [ ] actually fix the interactive app styles to not use the blocky look
- [ ] configurable interactive prompt (option to hide the quick select helper text, set vim mode)
- [ ] couple more flags 
  - `--include-remotes` / `-r` to interactive, direct jump, and list
  - `--vim-mode` to interactive


## Configuration

```toml
[general]
vim_mode = false
sources = ["local"] # "include_remotes" (maybe even configurable as to which remotes to include)

[appearrance]
quick_select_hint = "full" # or "compact" or "hidden"
theme = "default" # theme support comes in later
```

## Before release

- [ ] write tests
- [ ] remove debug clause from interactive app
- [ ] decide where to do the semver check
- [ ] update license
- [ ] update readme
- [ ] update name
- [ ] update help messages
- [ ] open issue upstream
- [ ] unlink from fork network



1. `git remote` get the list of remotes
2. `git ls-remote --heads <remote>` for each one, get the live branches

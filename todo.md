# Todo
- [ ] actually fix the interactive app styles to not use the blocky look
- [x] configurable interactive prompt (option to hide the quick select helper text, set vim mode)
- [x] couple more flags 
  - `--include-remotes` / `-r` to list (haven't figured out a nice way to do it in interactive mode yet)
  - `--vim-mode` to interactive


## Configuration

```toml
[general]
vim_mode = false
sources = ["local"] # "include_remotes" (maybe even configurable as to which remotes to include)

[appearance]
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

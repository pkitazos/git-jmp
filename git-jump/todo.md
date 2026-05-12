# Todo
- [ ] actually fix the interactive app styles to not use the blocky look
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

- [ ] remove debug clause from interactive app
- [ ] update license
- [ ] update readme
- [ ] update name
- [ ] update help messages
- [ ] open issue upstream
- [ ] unlink from fork network

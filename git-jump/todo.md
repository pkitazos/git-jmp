# Todo

- [x] display errors nicely
  - [ ] `rm` should not lose native git command info in its output `Deleted branch <branch name> (was <sha>).`
  - [ ] `rm` should remove newline char between multiple failure messages
  - [ ] `new` should also not lose native git command info `Switched to a new branch '<branch name>'`
  - [ ] `mv` is broken
- [ ] `git jump -` should work like `git switch -` it should not fuzzy match on some branch
- [ ] should be able to call `--version / -v` and `--help / -h` from anywhere on the system (probably comes by default with clap)
- [ ] finish wiring up interactive mode
  - [ ] in interactive mode you should be able to actually press `Enter` to jump to a branch (literally the whole point)
  - [ ] if you're hovering a worktree entry, `Enter` should cd you into that directory (maybe we add a little status message about this)
  - [ ] if you're in normal mode and switch to interactive mode the highlighted line should reset (maybe)
  - [ ] actually fix the styles to not use the blocky look
- [ ] configurable interactive prompt (option to hide the quick select helper text, set vim mode)

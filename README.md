![git-jump CLI logo](https://raw.githubusercontent.com/pkitazos/git-jump/main/img/readme-banner.png)

# Git-Jmp

Interactive navigation between branches and worktrees.

(heavily inspired from [mykolaharmash/git-jump](https://github.com/mykolaharmash/git-jump))

## Install

```shell
cargo install git-jmp
```

or using Homebrew

```shell
brew install git-jmp
```

## Usage

The tool has to main modes, the interactive switcher and the fuzzy match jump. 

### Interactive Mode

Run without arguments to launch the interactive UI:

```shell
git jmp
```

* When you first start using `git jmp` branches will just be sorted alphabetically, but as you start switching around using `git jmp` your jump history is tracked and the list will be sorted with the most recently jumped-to branches near the top of the list for faster switching.
* You can navigate the list with your arrow keys or, if vim mode is enabled, using `j/k`. Just hit enter to switch to the selected branch.
* You can filter the list using a fuzzy search, just start typing out a part of the name of the branch you want to jump to and you can narrow down the list.
* You can also quickly jump to any of your top 10 most recently visited branches using <kbd>Option</kbd>+<kbd>\<number\></kbd>. You might need to configure your terminal settings for the quick jump to work, see [section] below for how to set that up in some popular terminal.

### Direct Jump

If you know where you're going you can just jump there directly without going through the interactive UI. You can write the exact name of the branch or a partial match and `git-jmp` will do its best to get you there. First checking for an exact match and then falling back to a best fuzzy match. If you use feature branches with unique issue numbers, switching between feature branches is now very quick!

```shell
git jmp <fuzzy match>
```

Really, those are the two commands that make this tool useful to me, but for completeness there are a couple more sub-commands for common branch operations that are essentially wrappers over the native git commands which also record and update the jump data.

### new

```shell
git jmp new <branch name>
```

Runs `git switch --create` under the hood and also makes the new branch the most recently visited branch.

### mv

```shell
git jmp mv [<old name>] <new name>
```

Runs `git branch --move` under the hood and also updates the jump data with the new name.

### rm

```shell
git jmp rm <branch name> [<branch name>, ...]
```

Runs `git branch -d` for each of the branches provided and removes that entry from the jump data if the branch was successfully deleted.


### ls

```shell
git jmp ls
```

This one doesn't actually run `git branch`, but it does render the same list. The difference is that if you pipe the output from `ls` into some other command, the formatting is stripped unlike the native `git branch` which keeps the `*` and `+` identifiers. Idk why you would want this, but it's here if you want it?

## Configuration

You can configure `git-jmp` globally, locally per-project or by passing in flags to given commands. As of right now these are the supported configurations:

```toml
[general]
auto_check_updates = true
vim_mode = false

[appearance]
quick_select_hint = "full" # or "compact" or "hidden"
```

* `auto_check_updates` is pretty self-explanatory. Since checking for updates requires a network trip, if you'd rather not have `git-jmp` do that, you can just disable this.

The other two settings relate to the interactive mode of the tool explained above:
* `vim_mode` splits the list into two modes: `Normal` where you can just navigate around the interactive list, and `Input` which let's you edit the search input.
* `quick_select_hint` refers to the hint which is shown on the end of the search input line which explains how you can quick-jump to a particular branch: `"full"` is the verbose default that you can see in the example gif above, `"compact"` only includes the modifier key and number, and `"hidden"` shows nothing.

## Migrating from [mykolaharmash/git-jump](https://github.com/mykolaharmash/git-jump)

You can just install `git-jmp` and drop it into any project you previously used with `git-jump` and your jump data will be carried over. So in that senes this can act as a drop-in replacement. 
And you can still use `new`, but `rename`, `delete`, and `--list` have been replaced with shorter names.

I tried to cover all the issues from the original repo, and most fixes have been included in this version, I'm still working out what the nicest way to do remote branch support is. That should land soon enough.

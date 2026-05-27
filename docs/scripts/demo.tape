# git-jmp demo
# Prerequisites:
#   1. Run setup-demo.sh to create the demo repo
#   2. Ensure git-jmp is on $PATH (cargo install --path .)
#   3. Run from anywhere — the tape cd's into the demo repo

Require git-jmp

Output demo.gif

Set Shell "zsh"
Set FontSize 16
Set Width 1000
Set Height 600
Set Padding 20
Set Framerate 50
Set TypingSpeed 60ms
Set Theme "TokyoNight"


# --- 0. Hidden Setup

Hide
Type "cd /tmp/git-jmp-demo/demo"
Enter
Sleep 500ms
Type `setopt INTERACTIVE_COMMENTS`
Enter
Sleep 300ms
Type `eval "$(git-jmp init zsh)"`
Enter
Sleep 300ms
Type `precmd() { PS1="%F{blue}%1~%f %F{green}($(git branch --show-current 2>/dev/null))%f %F{white}❯%f " }`
Enter
Sleep 300ms
Type "clear"
Enter
Sleep 500ms
Show


# --- 1. List branches

Type "git jmp ls"
Sleep 600ms
Enter
Sleep 3s


# --- 2. Interactive mode with fuzzy search

Type "git jmp"
Sleep 600ms
Enter
Sleep 2.5s

Type "sign"
Sleep 2s

Down
Sleep 700ms
Enter
Sleep 2.5s


# --- 3. Direct fuzzy jump

Type "# jump directly with a partial match"
Sleep 500ms
Enter
Sleep 400ms

Type "git jmp 481"
Sleep 600ms
Enter
Sleep 2.5s


# --- 4. Jump back to previous branch

Type "# jump back to previous branch"
Sleep 500ms
Enter
Sleep 400ms

Type "git jmp -"
Sleep 600ms
Enter
Sleep 2.5s


# --- 5. Interactive mode with remote branches

Type "# include remote branches"
Sleep 500ms
Enter
Sleep 400ms

Type "git jmp -r"
Sleep 600ms
Enter
Sleep 2.5s

Down 5
Sleep 1.2s

Down 3
Sleep 700ms
Enter
Sleep 2.5s


# --- 6. Worktree navigation with shell integration

Type "# with shell integration, jump into worktrees"
Sleep 500ms
Enter
Sleep 400ms

Type "jmp"
Sleep 600ms
Enter
Sleep 2.5s

Down 9
Sleep 1s
Enter
Sleep 3s

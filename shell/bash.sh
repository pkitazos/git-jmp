jmp() {
    local tmpfile=$(mktemp)
    GIT_JMP_SHELL_INTEGRATION="$tmpfile" command git-jmp "$@"
    local exit_code=$?

    if [ $exit_code -eq 3 ]; then
        cd "$(cat "$tmpfile")"
    fi
    rm -f "$tmpfile"
}

source <(COMPLETE=bash git-jmp)

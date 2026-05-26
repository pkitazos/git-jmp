function jmp
    set tmpfile (mktemp)
    env GIT_JMP_SHELL_INTEGRATION="$tmpfile" command git-jmp $argv
    set exit_code $status

    if test $exit_code -eq 3
        cd (cat $tmpfile)
    end
    rm -f $tmpfile
end

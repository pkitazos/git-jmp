#!/bin/bash
set -e

DEMO_DIR="/tmp/git-jmp-demo"

# clean up any previous run
rm -rf "$DEMO_DIR"

mkdir -p "$DEMO_DIR" && cd "$DEMO_DIR"
git init demo && cd demo
git commit --allow-empty -m "initial commit"

# local branches
branches=(
  "dev"
  "feat/142-signup"
  "feat/481-auth-refactor"
  "feat/305-dashboard"
  "feat/519-onboarding"
  "fix/navbar-overflow"
  "chore/update-deps"
)

for b in "${branches[@]}"; do
  git branch "$b"
done

# worktrees
git branch hotfix/payments
git worktree add ../hotfix-payments hotfix/payments

git branch review/pr-287
git worktree add ../pr-287 review/pr-287

# fake remote
git init --bare "$DEMO_DIR/fake-origin"
git remote add origin "$DEMO_DIR/fake-origin"

# push all local branches so the remote has them
git push origin --all

# create remote-only branches (push then delete locally)
remote_only=(
  "feat/603-rate-limiting"
  "feat/774-dark-mode"
  "fix/820-ws-leak"
)

for b in "${remote_only[@]}"; do
  git branch "$b"
  git push origin "$b"
  git branch -D "$b"
done

# fetch so local cache has the remote refs
git fetch origin


mkdir -p .jump

now=$(date +%s)

cat > .jump/data.json << EOF
{
  "main": $((now - 30)),
  "dev": $((now - 120)),
  "feat/142-signup": $((now - 3600)),
  "feat/305-dashboard": $((now - 86400)),
  "feat/481-auth-refactor": $((now - 172800)),
  "feat/519-onboarding": $((now - 259200)),
  "fix/navbar-overflow": $((now - 345600)),
  "chore/update-deps": $((now - 604800))
}
EOF


git switch main

echo ""
echo "Demo repo ready at $DEMO_DIR/demo"
echo ""
echo "Before recording, make sure:"
echo "  1. git-jmp is installed:  cargo install --path /path/to/git-jump/git-jump"
echo "  2. Shell integration is sourced (for the jmp wrapper)"
echo "  3. cd $DEMO_DIR/demo"

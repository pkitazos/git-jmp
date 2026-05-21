# Release Instructions

## Publish a new version

1. Update `CHANGELOG.md`
2. Bump the version in `Cargo.toml`
3. Commit (e.g. `chore: release v0.2.0`)
4. Tag: `git tag v0.2.0`
5. `git push origin HEAD --tags`

## Create a GitHub release

1. Go to GitHub and create a new release based on the new tag
2. Describe what has changed in the new version (you can simply copy the relevant entry from `CHANGELOG.md`)

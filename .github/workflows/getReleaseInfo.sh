version=$(grep version Cargo.toml -m 1 | cut -d'=' -f2 | tr -d "\r" | tr -d ' ' | tr -d '"' | tr -d "'")
echo "version=$version">>$GITHUB_ENV
release_info=release.md
echo "release_info=$release_info">>$GITHUB_ENV
echo "## Changes" > $release_info
number=$(git log --oneline $(git rev-parse `git describe --tags --abbrev=0`)..HEAD | wc -l)
echo "$(git log --pretty='- [%h] %s' -$number)" >> $release_info

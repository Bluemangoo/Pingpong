version=$(grep version Cargo.toml -m 1 | cut -d'=' -f2 | tr -d "\r" | tr -d ' ' | tr -d '"' | tr -d "'")
echo $version
release_info=release.md
echo $release_info
echo "## Changes" > $release_info
number=$(git log --oneline master ^`git describe --tags --abbrev=0` | wc -l)
echo $number
echo "$(git log --pretty='> [%h] %s' -$number)" >> $release_info

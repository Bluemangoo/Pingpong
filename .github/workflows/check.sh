version=$(grep version Cargo.toml -m 1 | cut -d'=' -f2 | tr -d "\r" | tr -d ' ' | tr -d '"' | tr -d "'")
git rev-parse $version || exit 0
exit 1

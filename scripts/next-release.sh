#!/bin/bash

current_version=$(grep '^version =' Cargo.toml | sed -E 's/version = "(.*)"/\1/')

head=$(echo "$current_version" | cut -d '.' -f 1)
current_yearweek=$(echo "$current_version" | cut -d '.' -f 2)

yearweek=$(date +"%y%U")

if [[ "$current_yearweek" == "$yearweek" ]]; then
  current_build=$(echo "$current_version" | cut -d '.' -f 3)
  build=$(($current_build + 1))
else
  build=0
fi

new_version="$head.$yearweek.$build"

sed -i.bak "s/version = \"$current_version\"/version = \"$new_version\"/" Cargo.toml

cargo check

git add Cargo.toml Cargo.lock
git commit -m "Release version $new_version"

git tag "v$new_version"

git push origin main
git push origin "v$new_version"

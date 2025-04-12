#! /usr/bin/env bash

set -e

# Get the metadata for the workspace
metadata=$(cargo metadata --format-version=1 --no-deps)

# Extract the paths of the workspace members using jq
member_paths=$(echo "$metadata" | jq -r '.packages[] | select(.source == null) | .manifest_path' | xargs -I {} dirname {})
game_list=""

rm -rf _site
mkdir -p _site

# Loop through each member path and build it
for path in $member_paths; do
    echo "Building $path"
    game=$(basename "$path")
    game_list="$game_list\n    <li><a href="/$game"/>$game</li>"
    # Change to the member directory
    cd "$path"
    # Build the member
    trunk build --public-url "/$game"
    # Change back to the original directory
    cd -
    # Copy the built files to the _site directory
    # Assuming the built files are in the `dist` directory
    mkdir -p _site/$game
    cp -r "$path/dist"/* _site/$game
done

cp public/index.html _site
sed -i "s|GAME_LIST|$game_list|" _site/index.html
cp public/styles.css _site

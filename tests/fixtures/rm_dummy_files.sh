#!/bin/sh

# clean the project repo
repo_dir=$(CDPATH= cd "$(dirname "$0")/../.." && pwd)

for path in "$repo_dir/.venv" "$repo_dir/ishowspeed.mp4"; do
    if [ -L "$path" ]; then
        rm -- "$path"
    fi
done

# clean the dram repo
find /dev/shm/buas -mindepth 1 -maxdepth 1 -type d -exec rm -rf -- {} +

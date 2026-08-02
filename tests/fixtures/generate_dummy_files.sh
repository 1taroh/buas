#!/bin/sh

set -eu

usage() {
    echo "Usage: $0 [OUTPUT_DIR [FILE_COUNT [FILE_SIZE [DIRECTORY_COUNT]]]]" >&2
    echo "Defaults: OUTPUT_DIR=.venv, FILE_COUNT=1000, FILE_SIZE=4096, DIRECTORY_COUNT=20" >&2
}

case ${1-} in
    -h|--help)
        usage
        exit 0
        ;;
esac

output_dir=${1:-.venv}
file_count=${2:-100}
file_size=${3:-128}
directory_count=${4:-20}

if [ "$#" -gt 4 ]; then
    usage
    exit 2
fi

for value in "$file_count" "$file_size" "$directory_count"; do
    case $value in
        ''|*[!0-9]*)
            echo "error: numeric arguments must be non-negative integers" >&2
            usage
            exit 2
            ;;
    esac
done

if [ "$directory_count" -eq 0 ]; then
    echo "error: DIRECTORY_COUNT must be greater than zero" >&2
    exit 2
fi

mkdir -p "$output_dir"

i=0
while [ "$i" -lt "$file_count" ]; do
    directory="$output_dir/lib/package-$((i % directory_count))"
    mkdir -p "$directory"
    file="$directory/file-$i.bin"
    if [ "$file_size" -eq 0 ]; then
        : > "$file"
    else
        dd if=/dev/zero of="$file" bs="$file_size" count=1 2>/dev/null
    fi
    i=$((i + 1))
done

echo "generated $file_count files of $file_size bytes under $output_dir"

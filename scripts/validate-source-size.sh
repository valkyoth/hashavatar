#!/usr/bin/env sh
set -eu

maximum_lines=500
status=0
file_list="$(mktemp)"
trap 'rm -f "$file_list"' EXIT

find src crates tests examples fuzz/fuzz_targets \
    -type f -name '*.rs' -print | sort > "$file_list"

while IFS= read -r file; do
    lines="$(wc -l < "$file")"
    if [ "$lines" -gt "$maximum_lines" ]; then
        echo "source size: $file has $lines lines; maximum is $maximum_lines" >&2
        status=1
    fi
done < "$file_list"

if [ "$status" -ne 0 ]; then
    exit "$status"
fi

echo "source size: every Rust source file is at most $maximum_lines lines"

#!/usr/bin/env sh
set -eu

test -s docs/PANIC_POLICY.md

check_file() {
    file="$1"
    awk '
        /^#\[cfg\(test\)\]/ {
            exit failed
        }
        /^[[:space:]]*\/\/[!\/]/ {
            next
        }
        /assert!\(|panic!\(|unreachable!\(|\.unwrap\(|\.expect\(/ {
            allowed = 0
            if ($0 ~ /unreachable!\("SVG is handled outside AvatarOutputFormat"\)/) {
                allowed = 1
            }
            if (!allowed) {
                printf "panic policy: unreviewed panic-like site in %s:%d: %s\n", FILENAME, FNR, $0 > "/dev/stderr"
                failed = 1
            }
        }
        END {
            exit failed
        }
    ' "$file"
}

check_file src/lib.rs

echo "panic policy: ok"

#!/usr/bin/env sh
set -eu

find README.md CHANGELOG.md SECURITY.md docs release-notes security \
    -type f -name '*.md' -print \
    | while IFS= read -r file; do
        sed -n 's/.*](\([^)]*\.md[^)]*\)).*/\1/p' "$file" \
            | while IFS= read -r link; do
                case "$link" in
                    http://* | https://*)
                        continue
                        ;;
                esac

                target_link="${link%%#*}"
                case "$target_link" in
                    /*) target=".${target_link}" ;;
                    *) target="$(dirname "$file")/${target_link}" ;;
                esac

                if [ ! -f "$target" ] && [ ! -d "$target" ]; then
                    echo "missing Markdown link target: $file -> $link" >&2
                    exit 1
                fi
            done
    done

echo "documentation links: ok"

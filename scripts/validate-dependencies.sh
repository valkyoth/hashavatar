#!/usr/bin/env sh
set -eu

metadata="$(
    cargo metadata --no-deps --format-version 1
)"

for dependency in blake3 image palette rand sha2 subtle xxhash-rust zeroize; do
    case "$metadata" in
        *'"name":"'"$dependency"'"'*)
            ;;
        *)
            echo "dependency policy: missing expected direct dependency $dependency" >&2
            exit 1
            ;;
    esac
done

for forbidden in axum tokio hyper tower reqwest openssl actix-web rocket warp; do
    case "$metadata" in
        *'"name":"'"$forbidden"'"'*)
            echo "dependency policy: $forbidden does not belong in the reusable crate" >&2
            exit 1
            ;;
    esac
done

tree_root="$(
    cargo tree -e normal --depth 0
)"
case "$tree_root" in
    "hashavatar v"*) ;;
    *)
        echo "dependency policy: unexpected cargo tree root: $tree_root" >&2
        exit 1
        ;;
esac

echo "dependency policy: ok"

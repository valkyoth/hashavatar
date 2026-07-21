#!/usr/bin/env sh
set -eu

metadata="$(
    cargo metadata --no-deps --format-version 1
)"

for dependency in sanitization sanitization-crypto-interop roxmltree; do
    case "$metadata" in
        *'"name":"'"$dependency"'"'*)
            ;;
        *)
            echo "dependency policy: missing expected direct dependency $dependency" >&2
            exit 1
            ;;
    esac
done

for forbidden in axum tokio hyper tower reqwest openssl actix-web rocket warp image rand serde_json; do
    case "$metadata" in
        *'"name":"'"$forbidden"'"'*)
            echo "dependency policy: $forbidden does not belong in the reusable crate" >&2
            exit 1
            ;;
    esac
done

for package in hashavatar hashavatar-core; do
    if ! cargo tree -p "$package" -e normal --depth 0 | grep -q "^$package v"; then
        echo "dependency policy: missing workspace package $package" >&2
        exit 1
    fi
done

echo "dependency policy: ok"

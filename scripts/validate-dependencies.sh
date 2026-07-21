#!/usr/bin/env sh
set -eu

metadata="$(
    cargo metadata --no-deps --format-version 1
)"

for dependency in image sanitization sanitization-crypto-interop roxmltree; do
    case "$metadata" in
        *'"name":"'"$dependency"'"'*)
            ;;
        *)
            echo "dependency policy: missing expected direct dependency $dependency" >&2
            exit 1
            ;;
    esac
done

for forbidden in axum tokio hyper tower reqwest openssl actix-web rocket warp rand serde_json; do
    case "$metadata" in
        *'"name":"'"$forbidden"'"'*)
            echo "dependency policy: $forbidden does not belong in the reusable crate" >&2
            exit 1
            ;;
    esac
done

for package in hashavatar hashavatar-core hashavatar-formats; do
    if ! cargo tree -p "$package" -e normal --depth 0 | grep -q "^$package v"; then
        echo "dependency policy: missing workspace package $package" >&2
        exit 1
    fi
done

if cargo tree -p hashavatar-core -e normal | grep -q '^.*image v'; then
    echo "dependency policy: image leaked into hashavatar-core" >&2
    exit 1
fi

if cargo tree -p hashavatar --no-default-features -e normal | grep -q '^.*image v'; then
    echo "dependency policy: image leaked into the codec-free facade" >&2
    exit 1
fi

echo "dependency policy: ok"

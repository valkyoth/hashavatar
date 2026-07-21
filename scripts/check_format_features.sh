#!/usr/bin/env sh
set -eu

echo "format features: codec-free builds"
cargo check -p hashavatar-core --no-default-features
cargo check -p hashavatar-formats --no-default-features
cargo check -p hashavatar --no-default-features

core_tree="$(cargo tree -p hashavatar-core -e normal)"
minimal_tree="$(cargo tree -p hashavatar-formats --no-default-features -e normal)"
case "$core_tree$minimal_tree" in
    *" image v"*|*" image-webp v"*|*" png v"*|*" gif v"*|*" zune-jpeg v"*)
        echo "format features: codec dependency leaked into a codec-free graph" >&2
        exit 1
        ;;
esac

check_profile() {
    feature="$1"
    required="$2"
    forbidden_a="$3"
    forbidden_b="$4"
    forbidden_c="$5"
    echo "format features: $feature"
    cargo check -p hashavatar-formats --no-default-features --features "$feature"
    cargo check -p hashavatar --no-default-features --features "$feature"
    tree="$(cargo tree -p hashavatar-formats --no-default-features --features "$feature" -e normal)"
    case "$tree" in
        *"$required"*)
            ;;
        *)
            echo "format features: $feature is missing $required" >&2
            exit 1
            ;;
    esac
    for forbidden in "$forbidden_a" "$forbidden_b" "$forbidden_c"; do
        case "$tree" in
            *"$forbidden"*)
                echo "format features: $feature unexpectedly includes $forbidden" >&2
                exit 1
                ;;
        esac
    done
}

check_profile webp "image-webp v" " png v" " gif v" "zune-jpeg v"
check_profile png " png v" "image-webp v" " gif v" "zune-jpeg v"
check_profile jpeg "zune-jpeg v" "image-webp v" " png v" " gif v"
check_profile gif " gif v" "image-webp v" " png v" "zune-jpeg v"

echo "format features: all established formats"
cargo check -p hashavatar-formats --all-features
cargo check -p hashavatar --all-features

echo "format features: ok"

#!/usr/bin/env sh
set -eu

package_name="$(
    sed -n 's/^name = "\([^"]*\)"/\1/p' Cargo.toml | sed -n '1p'
)"
cargo_version="$(
    sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | sed -n '1p'
)"
cargo_rust_version="$(
    sed -n 's/^rust-version = "\([^"]*\)"/\1/p' Cargo.toml | sed -n '1p'
)"
toolchain_version="$(
    sed -n 's/^channel = "\([^"]*\)"/\1/p' rust-toolchain.toml | sed -n '1p'
)"

if [ "$package_name" != "hashavatar" ]; then
    echo "release metadata: package name must be hashavatar" >&2
    exit 1
fi

if [ -z "$cargo_version" ]; then
    echo "release metadata: Cargo.toml package version is missing" >&2
    exit 1
fi

if [ -z "$cargo_rust_version" ]; then
    echo "release metadata: Cargo.toml rust-version is missing" >&2
    exit 1
fi

if [ "$toolchain_version" != "$cargo_rust_version.0" ]; then
    echo "release metadata: rust-toolchain.toml channel $toolchain_version does not match Cargo.toml rust-version $cargo_rust_version" >&2
    exit 1
fi

if ! grep -q '^license = "MIT OR Apache-2.0"$' Cargo.toml; then
    echo "release metadata: Cargo.toml must declare license = \"MIT OR Apache-2.0\"" >&2
    exit 1
fi

if ! grep -q '^repository = "https://github.com/valkyoth/hashavatar"$' Cargo.toml; then
    echo "release metadata: Cargo.toml repository must be https://github.com/valkyoth/hashavatar" >&2
    exit 1
fi

if ! grep -q '^homepage = "https://github.com/valkyoth/hashavatar"$' Cargo.toml; then
    echo "release metadata: Cargo.toml homepage must be https://github.com/valkyoth/hashavatar" >&2
    exit 1
fi

test -s LICENSE-MIT
test -s LICENSE-APACHE
test -s rust-toolchain.toml
test -s deny.toml
test -s README.md
test -s CONTRIBUTING.md
test -s SECURITY.md
test -s docs/DEPENDENCIES.md
test -s docs/PANIC_POLICY.md
test -s docs/RELEASE.md
test -s docs/SECURITY_CONTROLS.md
test -s "docs/release-notes/RELEASE_NOTES_${cargo_version}.md"

for required_script in \
    "scripts/check_fuzz.sh" \
    "scripts/checks.sh" \
    "scripts/generate-sbom.sh" \
    "scripts/reproducible_build_check.sh" \
    "scripts/stable_release_gate.sh" \
    "scripts/validate-dependencies.sh" \
    "scripts/validate-panic-policy.sh" \
    "scripts/validate-release-metadata.sh" \
    "scripts/validate-unsafe-boundary.sh"
do
    if [ ! -x "$required_script" ]; then
        echo "release metadata: $required_script must be executable" >&2
        exit 1
    fi

    if [ "$(sed -n '1p' "$required_script")" != "#!/usr/bin/env sh" ]; then
        echo "release metadata: $required_script must use #!/usr/bin/env sh" >&2
        exit 1
    fi
done

if ! grep -q '^The MIT License (MIT)$' LICENSE-MIT; then
    echo "release metadata: LICENSE-MIT does not look like the canonical MIT license" >&2
    exit 1
fi

if ! grep -q 'Apache License' LICENSE-APACHE || ! grep -q 'Version 2.0, January 2004' LICENSE-APACHE; then
    echo "release metadata: LICENSE-APACHE does not look like the canonical Apache 2.0 license" >&2
    exit 1
fi

if ! grep -q "^## $cargo_version" CHANGELOG.md; then
    echo "release metadata: CHANGELOG.md is missing a section for Cargo version $cargo_version" >&2
    exit 1
fi

package_list="$(
    cargo package --locked --allow-dirty --list
)"

for required_package_file in \
    "CHANGELOG.md" \
    "CONTRIBUTING.md" \
    "Cargo.lock" \
    "Cargo.toml" \
    "deny.toml" \
    "docs/DEPENDENCIES.md" \
    "docs/PANIC_POLICY.md" \
    "docs/RELEASE.md" \
    "docs/SECURITY_CONTROLS.md" \
    "LICENSE-APACHE" \
    "LICENSE-MIT" \
    "README.md" \
    "docs/release-notes/RELEASE_NOTES_${cargo_version}.md" \
    "rust-toolchain.toml" \
    "SECURITY.md" \
    "scripts/check_fuzz.sh" \
    "scripts/checks.sh" \
    "scripts/generate-sbom.sh" \
    "scripts/reproducible_build_check.sh" \
    "scripts/stable_release_gate.sh" \
    "scripts/validate-dependencies.sh" \
    "scripts/validate-panic-policy.sh" \
    "scripts/validate-release-metadata.sh" \
    "scripts/validate-unsafe-boundary.sh" \
    "src/lib.rs" \
    "tests/golden_fingerprints.txt"
do
    if ! printf '%s\n' "$package_list" | grep -qx "$required_package_file"; then
        echo "release metadata: package is missing $required_package_file" >&2
        exit 1
    fi
done

if printf '%s\n' "$package_list" | grep -q '^fuzz/'; then
    echo "release metadata: fuzz-only harness files must not be included in the published crate" >&2
    exit 1
fi

if printf '%s\n' "$package_list" | grep -q '^src/main.rs$'; then
    echo "release metadata: bundled demo server must not be included in the published crate" >&2
    exit 1
fi

if printf '%s\n' "$package_list" | grep -q '^src/bin/'; then
    echo "release metadata: binary targets must not be included in the pure library crate" >&2
    exit 1
fi

echo "release metadata: ok"

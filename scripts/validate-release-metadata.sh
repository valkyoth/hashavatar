#!/usr/bin/env sh
set -eu

metadata="$(cargo metadata --no-deps --format-version 1)"
package_version() {
    package="$1"
    printf '%s' "$metadata" | python3 -c '
import json, sys
name = sys.argv[1]
data = json.load(sys.stdin)
print(next(package["version"] for package in data["packages"] if package["name"] == name))
' "$package"
}

facade_version="$(package_version hashavatar)"
core_version="$(package_version hashavatar-core)"
toolchain_version="$(sed -n 's/^channel = "\([^"]*\)"/\1/p' rust-toolchain.toml | sed -n '1p')"

if [ "$facade_version" != "2.0.0-alpha.1" ]; then
    echo "release metadata: facade must be 2.0.0-alpha.1, got $facade_version" >&2
    exit 1
fi
if [ "$core_version" != "0.1.0-alpha.1" ]; then
    echo "release metadata: core must be 0.1.0-alpha.1, got $core_version" >&2
    exit 1
fi
if [ "$toolchain_version" != "1.97.1" ]; then
    echo "release metadata: development toolchain must be 1.97.1" >&2
    exit 1
fi
if ! grep -q '^rust-version = "1.90"$' Cargo.toml; then
    echo "release metadata: workspace MSRV must remain Rust 1.90" >&2
    exit 1
fi
if ! grep -q '^resolver = "3"$' Cargo.toml; then
    echo "release metadata: workspace must use resolver 3" >&2
    exit 1
fi
if ! grep -q '^license = "MIT OR Apache-2.0"$' Cargo.toml; then
    echo "release metadata: workspace license must be MIT OR Apache-2.0" >&2
    exit 1
fi

for required_file in \
    CHANGELOG.md \
    Cargo.lock \
    Cargo.toml \
    LICENSE-APACHE \
    LICENSE-MIT \
    README.md \
    SECURITY.md \
    crates/hashavatar-core/Cargo.toml \
    crates/hashavatar-core/README.md \
    deny.toml \
    docs/CONTRIBUTING.md \
    docs/CRATE_VERSION_MATRIX.md \
    docs/CURRENT_STATUS.md \
    docs/DEPENDENCIES.md \
    docs/KANI.md \
    docs/MIGRATION_2.0.md \
    docs/PANIC_POLICY.md \
    docs/PLAN_TOWARDS_2.0.md \
    docs/PROVENANCE.md \
    docs/README_POLICY.md \
    docs/RELEASE.md \
    docs/SECURITY_CONTROLS.md \
    docs/STABILITY.md \
    docs/THIRD_PARTY_NOTICES.md \
    docs/VERSIONING.md \
    release-crates.toml \
    release-notes/RELEASE_NOTES_2.0.0-alpha.1.md \
    rust-toolchain.toml \
    security/pentest/README.md
do
    test -s "$required_file"
done

if [ -e PENTEST.md ]; then
    echo "release metadata: root PENTEST.md is temporary and must be removed" >&2
    exit 1
fi
if ! grep -q '^## 2.0.0-alpha.1$' CHANGELOG.md; then
    echo "release metadata: changelog is missing alpha.1" >&2
    exit 1
fi
if ! grep -q 'version = "2.0.0-alpha.1"' release-crates.toml; then
    echo "release metadata: release plan is not alpha.1" >&2
    exit 1
fi
if grep -q '^publish = true$' release-crates.toml; then
    echo "release metadata: prerelease crates must not be published" >&2
    exit 1
fi

kani_source_count="$(grep -c '^#\[kani::proof\]$' crates/hashavatar-core/src/kani_proofs.rs)"
kani_documented_count="$(
    sed -n 's/^- Current admitted harness count: `\([0-9][0-9]*\)`\.$/\1/p' docs/KANI.md
)"
if [ "$kani_source_count" != "$kani_documented_count" ]; then
    echo "release metadata: Kani source and documented counts differ" >&2
    exit 1
fi

for script in \
    scripts/check_doc_links.sh \
    scripts/check_fuzz.sh \
    scripts/check_kani.sh \
    scripts/checks.sh \
    scripts/generate-sbom.sh \
    scripts/reproducible_build_check.sh \
    scripts/stable_release_gate.sh \
    scripts/validate-dependencies.sh \
    scripts/validate-panic-policy.sh \
    scripts/validate-release-metadata.sh \
    scripts/validate-source-size.sh \
    scripts/validate-unsafe-boundary.sh
do
    if [ ! -x "$script" ] || [ "$(sed -n '1p' "$script")" != "#!/usr/bin/env sh" ]; then
        echo "release metadata: $script must be executable POSIX shell" >&2
        exit 1
    fi
done

for script in scripts/release_crates.py scripts/test_release_crates.py; do
    if [ ! -x "$script" ] || [ "$(sed -n '1p' "$script")" != "#!/usr/bin/env python3" ]; then
        echo "release metadata: $script must be executable Python" >&2
        exit 1
    fi
done

if ! grep -q 'expected_kani="cargo-kani 0.67.0"' scripts/check_kani.sh \
    || ! grep -q 'expected_sbom="cargo-sbom 0.10.0"' scripts/generate-sbom.sh
then
    echo "release metadata: assurance tool pins changed" >&2
    exit 1
fi

facade_files="$(cargo package -p hashavatar --locked --allow-dirty --list)"
core_files="$(cargo package -p hashavatar-core --locked --allow-dirty --list)"
for file in Cargo.toml README.md src/lib.rs; do
    if ! printf '%s\n' "$facade_files" | grep -qx "$file"; then
        echo "release metadata: facade archive is missing $file" >&2
        exit 1
    fi
done
for file in Cargo.toml README.md src/lib.rs src/cat.rs src/raster.rs src/scene.rs src/svg.rs; do
    if ! printf '%s\n' "$core_files" | grep -qx "$file"; then
        echo "release metadata: core archive is missing $file" >&2
        exit 1
    fi
done
if printf '%s\n%s\n' "$facade_files" "$core_files" | grep -q '^fuzz/'; then
    echo "release metadata: fuzz files must not ship in crate archives" >&2
    exit 1
fi

echo "release metadata: ok"

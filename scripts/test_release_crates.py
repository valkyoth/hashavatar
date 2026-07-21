#!/usr/bin/env python3
"""Self-tests for the Hashavatar workspace release policy."""

from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "release_crates.py"


def load_module():
    spec = importlib.util.spec_from_file_location("release_crates", SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load release_crates.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


release_crates = load_module()


def assert_fails(expected: str, function, *args, **kwargs) -> None:
    try:
        function(*args, **kwargs)
    except RuntimeError as error:
        if expected not in str(error):
            raise AssertionError(f"expected {expected!r} in {error!r}") from error
        return
    raise AssertionError("expected RuntimeError")


def entry(*, previous: str, version: str, change: str, publish: bool) -> dict:
    return {
        "previous_version": previous,
        "version": version,
        "change": change,
        "publish": publish,
        "reason": "test fixture",
    }


def package(name: str, version: str, dependencies: tuple[str, ...] = ()) -> dict:
    return {
        "name": name,
        "version": version,
        "publish": None,
        "dependencies": [{"name": dependency, "kind": None} for dependency in dependencies],
    }


def test_prerelease_detection() -> None:
    assert not release_crates.is_prerelease("2.0.0")
    assert release_crates.is_prerelease("2.0.0-alpha.1")
    assert release_crates.is_prerelease("2.0.0-rc.2")


def test_prerelease_publication_is_rejected() -> None:
    candidate = entry(
        previous="1.3.0", version="2.0.0-alpha.1", change="code", publish=True
    )
    assert_fails(
        "commit-only",
        release_crates.validate_plan_entry,
        "hashavatar",
        candidate,
        prerelease=True,
    )


def test_prerelease_changed_crate_can_remain_unpublished() -> None:
    candidate = entry(
        previous="1.3.0", version="2.0.0-alpha.1", change="code", publish=False
    )
    release_crates.validate_plan_entry(
        "hashavatar", candidate, prerelease=True
    )


def test_stable_changed_crate_must_be_published() -> None:
    candidate = entry(
        previous="2.0.0-rc.2", version="2.0.0", change="code", publish=False
    )
    assert_fails(
        "stable release",
        release_crates.validate_plan_entry,
        "hashavatar",
        candidate,
        prerelease=False,
    )


def test_unchanged_crate_cannot_be_published() -> None:
    candidate = entry(
        previous="0.1.0", version="0.1.0", change="unchanged", publish=True
    )
    assert_fails(
        "unchanged but publish is true",
        release_crates.validate_plan_entry,
        "hashavatar-core",
        candidate,
        prerelease=False,
    )


def test_dependency_order_is_enforced() -> None:
    packages = {
        "hashavatar": package("hashavatar", "2.0.0", ("hashavatar-core",)),
        "hashavatar-core": package("hashavatar-core", "1.0.0"),
    }
    plan = {
        "publish_order": ("hashavatar", "hashavatar-core"),
        "crates": {
            "hashavatar": {"version": "2.0.0", "publish": True},
            "hashavatar-core": {"version": "1.0.0", "publish": True},
        },
    }
    assert_fails(
        "dependency hashavatar-core after hashavatar",
        release_crates.verify_workspace,
        packages,
        plan,
    )


def test_current_release_plan_matches_workspace() -> None:
    plan = release_crates.load_release_plan(ROOT / "release-crates.toml")
    packages = release_crates.workspace_packages(release_crates.cargo_metadata())
    release_crates.verify_workspace(packages, plan)


def test_source_only_facade_defers_archive_until_core_exists() -> None:
    packages = {
        "hashavatar-core": package("hashavatar-core", "0.1.0-alpha.1"),
        "hashavatar": package(
            "hashavatar", "2.0.0-alpha.1", ("hashavatar-core",)
        ),
    }
    plan = {
        "crates": {
            "hashavatar-core": entry(
                previous="0.0.0",
                version="0.1.0-alpha.1",
                change="code",
                publish=False,
            ),
            "hashavatar": entry(
                previous="1.3.0",
                version="2.0.0-alpha.1",
                change="code",
                publish=False,
            ),
        }
    }
    order = ("hashavatar-core", "hashavatar")
    assert not release_crates.needs_unpublished_workspace_dependency(
        "hashavatar-core", order, packages, plan
    )
    assert release_crates.needs_unpublished_workspace_dependency(
        "hashavatar", order, packages, plan
    )


def main() -> int:
    tests = sorted(
        (
            (name, value)
            for name, value in globals().items()
            if name.startswith("test_") and callable(value)
        ),
        key=lambda item: item[0],
    )
    for _, test in tests:
        test()
    print(f"release script tests: {len(tests)} passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

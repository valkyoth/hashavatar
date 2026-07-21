#!/usr/bin/env python3
"""Validate and publish Hashavatar workspace crates in dependency order."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - release host guard.
    print("Python 3.11+ is required because this script uses tomllib.", file=sys.stderr)
    raise


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PLAN = ROOT / "release-crates.toml"
CHANGE_KINDS = ("code", "bugfix", "dependency", "metadata", "unchanged")
SEMVER = re.compile(
    r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"
    r"(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$"
)


def run(command: list[str], *, dry_run: bool) -> None:
    print(f"+ {' '.join(command)}", flush=True)
    if not dry_run:
        subprocess.run(command, cwd=ROOT, check=True)


def capture(command: list[str]) -> str:
    return subprocess.check_output(command, cwd=ROOT, text=True).strip()


def load_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def validate_version(version: object, *, field: str) -> str:
    if not isinstance(version, str) or SEMVER.fullmatch(version) is None:
        raise RuntimeError(f"{field} must be a complete SemVer version")
    return version


def is_prerelease(version: str) -> bool:
    match = SEMVER.fullmatch(version)
    if match is None:
        raise RuntimeError(f"invalid SemVer version: {version}")
    return match.group(4) is not None


def cargo_metadata() -> dict:
    return json.loads(capture(["cargo", "metadata", "--format-version", "1", "--no-deps"]))


def workspace_packages(metadata: dict) -> dict[str, dict]:
    workspace_ids = set(metadata["workspace_members"])
    return {
        package["name"]: package
        for package in metadata["packages"]
        if package["id"] in workspace_ids
    }


def load_release_plan(path: Path) -> dict:
    raw = load_toml(path)
    release = raw.get("release", {})
    crates = raw.get("crates", {})
    version = validate_version(release.get("version"), field="release.version")
    order = release.get("publish_order")
    if not isinstance(order, list) or not order or not all(isinstance(item, str) for item in order):
        raise RuntimeError("release.publish_order must be a non-empty string list")
    if len(order) != len(set(order)):
        raise RuntimeError("release.publish_order contains duplicate package names")
    if not isinstance(crates, dict) or set(crates) != set(order):
        raise RuntimeError("release plan crates must exactly match release.publish_order")

    prerelease = is_prerelease(version)
    for package_name, entry in crates.items():
        validate_plan_entry(package_name, entry, prerelease=prerelease)
    return {"version": version, "publish_order": tuple(order), "crates": crates}


def validate_plan_entry(package_name: str, entry: object, *, prerelease: bool) -> None:
    if not isinstance(entry, dict):
        raise RuntimeError(f"{package_name} release entry must be a table")
    previous = validate_version(entry.get("previous_version"), field=f"{package_name}.previous_version")
    version = validate_version(entry.get("version"), field=f"{package_name}.version")
    change = entry.get("change")
    publish = entry.get("publish")
    reason = entry.get("reason")
    if change not in CHANGE_KINDS:
        raise RuntimeError(f"{package_name} has invalid change kind {change!r}")
    if not isinstance(publish, bool):
        raise RuntimeError(f"{package_name}.publish must be true or false")
    if not isinstance(reason, str) or not reason.strip():
        raise RuntimeError(f"{package_name}.reason must explain the release decision")

    if change == "unchanged":
        if version != previous:
            raise RuntimeError(f"{package_name} is unchanged but its version changed")
        if publish:
            raise RuntimeError(f"{package_name} is unchanged but publish is true")
    elif version == previous:
        raise RuntimeError(f"{package_name} changed but its version did not")

    if prerelease and publish:
        raise RuntimeError(
            f"{package_name} enables crates.io publication for a prerelease; "
            "Hashavatar prereleases are commit-only milestones"
        )
    if not prerelease and change != "unchanged" and not publish:
        raise RuntimeError(f"{package_name} changed for a stable release but publish is false")


def verify_workspace(packages: dict[str, dict], plan: dict) -> None:
    order = plan["publish_order"]
    if set(packages) != set(order):
        raise RuntimeError(
            "release plan is not in sync with workspace packages: "
            f"workspace={tuple(sorted(packages))}, plan={tuple(sorted(order))}"
        )

    positions = {name: index for index, name in enumerate(order)}
    for name in order:
        package = packages[name]
        expected = plan["crates"][name]["version"]
        if package["version"] != expected:
            raise RuntimeError(f"{name} is version {package['version']}, expected {expected}")
        if package.get("publish") == [] and plan["crates"][name]["publish"]:
            raise RuntimeError(f"{name} is marked publish = false in Cargo metadata")

        for dependency in package["dependencies"]:
            dependency_name = dependency["name"]
            if dependency.get("kind") == "dev" or dependency_name not in positions:
                continue
            if positions[dependency_name] >= positions[name]:
                raise RuntimeError(
                    f"publish order places dependency {dependency_name} after {name}"
                )


def require_clean_tree() -> None:
    status = capture(["git", "status", "--porcelain"])
    if status:
        raise RuntimeError(f"refusing release operation from a dirty worktree:\n{status}")


def verify_no_scratch_pentest() -> None:
    if (ROOT / "PENTEST.md").exists():
        raise RuntimeError("root PENTEST.md is temporary input and must be removed")


def verify_pentest_report(version: str) -> None:
    report = ROOT / "security" / "pentest" / f"v{version}.md"
    if not report.is_file():
        raise RuntimeError(f"missing permanent pentest report: {report.relative_to(ROOT)}")
    content = report.read_text(encoding="utf-8")
    for field in (
        "Status: PASS",
        "Reviewed-Range: ",
        "Reviewed-Commit: ",
        "Tester: ",
        "Scope: ",
        "Date: ",
    ):
        if field not in content:
            raise RuntimeError(f"{report.relative_to(ROOT)} is missing {field.strip()}")


def verify_tag_at_head(version: str) -> None:
    tag = f"v{version}"
    head = capture(["git", "rev-parse", "HEAD"])
    result = subprocess.run(
        ["git", "rev-parse", "-q", "--verify", f"refs/tags/{tag}^{{commit}}"],
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(f"release tag {tag} does not exist")
    tagged = result.stdout.strip()
    if tagged != head:
        raise RuntimeError(f"HEAD {head} is not the commit tagged by {tag} ({tagged})")


def publish_plan(plan: dict) -> tuple[str, ...]:
    return tuple(
        name for name in plan["publish_order"] if plan["crates"][name]["publish"]
    )


def run_preflight(*, stable: bool, dry_run: bool) -> None:
    mode = "release" if stable else "check"
    run(["scripts/stable_release_gate.sh", mode], dry_run=dry_run)


def needs_unpublished_workspace_dependency(
    package: str, order: tuple[str, ...], packages: dict[str, dict], plan: dict
) -> bool:
    workspace_names = set(order)
    workspace_dependencies = {
        dependency["name"]
        for dependency in packages[package]["dependencies"]
        if dependency.get("kind") != "dev" and dependency["name"] in workspace_names
    }
    return any(
        plan["crates"][dependency]["change"] != "unchanged"
        or is_prerelease(plan["crates"][dependency]["version"])
        for dependency in workspace_dependencies
    )


def verify_packages(
    order: tuple[str, ...], packages: dict[str, dict], plan: dict, *, dry_run: bool
) -> None:
    for package in order:
        if needs_unpublished_workspace_dependency(package, order, packages, plan):
            run(
                [
                    "cargo",
                    "package",
                    "-p",
                    package,
                    "--locked",
                    "--allow-dirty",
                    "--list",
                ],
                dry_run=dry_run,
            )
            continue
        run(
            [
                "cargo",
                "package",
                "-p",
                package,
                "--locked",
                "--allow-dirty",
                "--no-verify",
            ],
            dry_run=dry_run,
        )


def wait_for_index(package: str, version: str, *, dry_run: bool) -> None:
    print(f"Published {package} {version}.")
    print(f"Wait for https://crates.io/crates/{package}/{version}, then press Enter.")
    if dry_run:
        print("[dry-run] skipping wait")
        return
    input()
    time.sleep(5)


def publish(package: str, *, dry_run: bool) -> None:
    run(["cargo", "publish", "-p", package, "--locked"], dry_run=dry_run)


def plan_path(raw: str) -> Path:
    path = Path(raw)
    return path if path.is_absolute() else (ROOT / path).resolve()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate or publish Hashavatar workspace crates in dependency order."
    )
    parser.add_argument("--plan", default=str(DEFAULT_PLAN))
    parser.add_argument("--version", default=None)
    parser.add_argument("--start-at", default=None)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--skip-checks", action="store_true")
    parser.add_argument("--require-tag", action="store_true")
    parser.add_argument("--yes", action="store_true")
    args = parser.parse_args()

    if args.check and args.prepare_only:
        parser.error("--check and --prepare-only are mutually exclusive")
    if args.prepare_only and args.require_tag:
        parser.error("--prepare-only cannot require a tag")

    plan = load_release_plan(plan_path(args.plan))
    if args.version is not None and args.version != plan["version"]:
        raise RuntimeError(
            f"--version {args.version} does not match release plan {plan['version']}"
        )
    args.version = plan["version"]
    packages = workspace_packages(cargo_metadata())
    verify_workspace(packages, plan)
    verify_no_scratch_pentest()

    if args.check:
        print(f"release plan {args.version}: workspace versions and order are valid")
        return 0

    require_clean_tree()
    prerelease = is_prerelease(args.version)
    if not args.skip_checks:
        run_preflight(stable=not prerelease, dry_run=args.dry_run)
    verify_packages(plan["publish_order"], packages, plan, dry_run=args.dry_run)

    if args.prepare_only:
        print(f"release preparation completed for {args.version}; nothing was uploaded")
        return 0

    if prerelease:
        raise RuntimeError(
            "crates.io publication is disabled for alpha, beta, and RC versions; "
            "test and record the exact implementation-stop commit instead"
        )
    if not args.require_tag:
        raise RuntimeError("stable publication requires explicit --require-tag")
    verify_pentest_report(args.version)
    verify_tag_at_head(args.version)

    selected = publish_plan(plan)
    if not selected:
        raise RuntimeError("release plan selects no crates for publication")
    if args.start_at is not None:
        if args.start_at not in selected:
            raise RuntimeError(f"--start-at package is not selected: {args.start_at}")
        selected = selected[selected.index(args.start_at) :]

    print(f"Stable release: {args.version}")
    print("Publish sequence:")
    for package in selected:
        print(f"  - {package} {plan['crates'][package]['version']}")
    if not args.yes:
        answer = input("Type the stable release version to publish: ").strip()
        if answer != args.version:
            raise RuntimeError("version confirmation did not match")

    for index, package in enumerate(selected):
        publish(package, dry_run=args.dry_run)
        if index + 1 < len(selected):
            wait_for_index(package, plan["crates"][package]["version"], dry_run=args.dry_run)

    print("Stable crates.io publish sequence completed.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as error:
        print(f"release error: {error}", file=sys.stderr)
        raise SystemExit(1) from error

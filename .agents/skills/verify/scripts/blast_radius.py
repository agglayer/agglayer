#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:
    yaml = None  # type: ignore[assignment]


PROSE_EXTENSIONS = {".md", ".adoc", ".rst", ".txt"}

DEFAULT_CONFIG: dict[str, Any] = {
    "core_crates": [],
    "risk_areas": [],
    "docs_commands": [],
    "code_commands": [],
}

RUNTIME_MARKER_FILES = {
    "Cargo.toml",
    "Cargo.lock",
    "Makefile.toml",
    "rust-toolchain",
    "rust-toolchain.toml",
}


def run_git(args: list[str]) -> str:
    completed = subprocess.run(
        ["git", *args],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"git {' '.join(args)} failed: {completed.stderr.strip()}")
    return completed.stdout.strip()


def git_ref_exists(ref: str) -> bool:
    completed = subprocess.run(
        ["git", "rev-parse", "--verify", "--quiet", ref],
        check=False,
        capture_output=True,
        text=True,
    )
    return completed.returncode == 0


def split_nonempty_lines(value: str) -> list[str]:
    return [line for line in value.splitlines() if line]


def load_config() -> dict[str, Any]:
    """Load ``.blast-radius.yaml`` from the repo root.

    Returns *DEFAULT_CONFIG* when the file is absent,
    unreadable, or PyYAML is not installed.
    """
    if yaml is None:
        print(
            "warning: PyYAML not installed; "
            "install with `pip install pyyaml` for config support. "
            "Using built-in defaults.",
            file=sys.stderr,
        )
        return dict(DEFAULT_CONFIG)

    try:
        repo_root = run_git(["rev-parse", "--show-toplevel"])
    except RuntimeError:
        return dict(DEFAULT_CONFIG)

    config_path = Path(repo_root) / ".blast-radius.yaml"
    if not config_path.exists():
        return dict(DEFAULT_CONFIG)

    try:
        with open(config_path) as fh:
            raw = yaml.safe_load(fh)
    except Exception:
        return dict(DEFAULT_CONFIG)

    if not isinstance(raw, dict):
        return dict(DEFAULT_CONFIG)

    return {
        "core_crates": raw.get("core_crates", []) or [],
        "risk_areas": raw.get("risk_areas", []) or [],
        "docs_commands": raw.get("docs_commands", []) or [],
        "code_commands": raw.get("code_commands", []) or [],
    }


def list_head_commit_files() -> tuple[list[str], str] | None:
    if git_ref_exists("HEAD^"):
        compared = split_nonempty_lines(run_git(["diff", "--name-only", "HEAD^..HEAD"]))
        return sorted(set(compared)), "HEAD^..HEAD"

    if git_ref_exists("HEAD"):
        compared = split_nonempty_lines(
            run_git(["show", "--pretty=format:", "--name-only", "HEAD"])
        )
        return sorted(set(compared)), "HEAD"

    return None


def list_changed_files() -> tuple[list[str], str]:
    staged = split_nonempty_lines(run_git(["diff", "--name-only", "--cached"]))
    unstaged = split_nonempty_lines(run_git(["diff", "--name-only"]))
    untracked = split_nonempty_lines(
        run_git(["ls-files", "--others", "--exclude-standard"])
    )
    local_changes = sorted(set(staged + unstaged + untracked))

    for baseline in ("main", "origin/main"):
        if git_ref_exists(baseline):
            compared = split_nonempty_lines(
                run_git(["diff", "--name-only", f"{baseline}...HEAD"])
            )
            changed_files = sorted(set(compared + local_changes))
            if local_changes:
                return changed_files, f"{baseline}...HEAD + working-tree"
            return changed_files, f"{baseline}...HEAD"

    head_changes = list_head_commit_files()
    if head_changes is not None:
        compared, source = head_changes
        changed_files = sorted(set(compared + local_changes))
        if local_changes:
            return changed_files, f"{source} + working-tree"
        return changed_files, source

    if local_changes:
        return local_changes, "working-tree"

    return [], "none"


def is_prose_or_docs(path: str) -> bool:
    file_path = Path(path)
    if path.startswith("docs/"):
        return True
    if file_path.suffix in PROSE_EXTENSIONS:
        return True
    return path in {
        "README",
        "README.md",
        "AGENTS.md",
        "CONTRIBUTING.md",
        "CHANGELOG.md",
        "LICENSE",
    }


def crate_name(path: str) -> str | None:
    if not path.startswith("crates/"):
        return None
    parts = path.split("/", 2)
    if len(parts) < 2:
        return None
    return parts[1]


def unique(values: list[str]) -> list[str]:
    return sorted(set(values))


def match_pattern(path: str, pattern: str) -> bool:
    """Check if *path* matches *pattern* using prefix matching.

    A pattern ending with ``/`` or ``*`` matches any path
    that starts with the prefix (stripping the trailing ``*``).
    A pattern without these suffixes also uses prefix matching
    so ``crates/foo`` matches ``crates/foo/src/lib.rs``.
    """
    prefix = pattern.rstrip("*")
    return path.startswith(prefix)


def parse_analysis(
    changed_files: list[str],
    analysis_source: str,
    config: dict[str, Any] | None = None,
) -> dict[str, Any]:
    if config is None:
        config = DEFAULT_CONFIG

    core_crates: set[str] = set(config.get("core_crates", []))
    risk_areas: list[dict[str, Any]] = config.get("risk_areas", [])
    docs_commands: list[str] = config.get("docs_commands", [])
    code_commands: list[str] = config.get("code_commands", [])

    # Discover affected crates from crates/ paths.
    crates = unique(
        [name for path in changed_files if (name := crate_name(path)) is not None]
    )

    knowledge_base_changed = any(
        path.startswith("docs/knowledge-base/") for path in changed_files
    )

    docs_only = bool(changed_files) and all(
        is_prose_or_docs(path) for path in changed_files
    )
    runtime_behavior_may_change = any(
        path.startswith("crates/")
        or path.startswith("proto/")
        or path.startswith("tests/")
        or Path(path).name in RUNTIME_MARKER_FILES
        for path in changed_files
    )

    # Match risk areas from config.
    affected_modules: set[str] = set(crates)
    risk_flags: list[str] = []
    triggered_scopes: set[str] = set()
    scope_commands: dict[str, list[str]] = {}

    for area in risk_areas:
        area_name: str = area["name"]
        patterns: list[str] = area.get("patterns", [])
        matched = any(
            match_pattern(path, pattern)
            for path in changed_files
            for pattern in patterns
        )
        if not matched:
            continue

        risk_flags.append(area_name)

        if "propagates_to" in area:
            affected_modules.update(area["propagates_to"])

        if "scope" in area:
            scope = area["scope"]
            triggered_scopes.add(scope)
            if "commands" in area:
                scope_commands.setdefault(scope, []).extend(area["commands"])

    # Build recommended scopes.
    recommended_scopes = ["minimal"]
    if runtime_behavior_may_change and not docs_only:
        recommended_scopes.append("code")
    for scope in sorted(triggered_scopes):
        if scope not in recommended_scopes:
            recommended_scopes.append(scope)

    # Broad impact detection.
    broad_impact = bool(
        len(affected_modules) >= 4
        or any(crate in core_crates for crate in affected_modules)
        or any(scope in triggered_scopes for scope in ("proto",))
    )

    # Build recommended commands.
    recommended_commands: list[str] = []

    # Scope-triggered commands that should run first (e.g. proto generation).
    for scope in sorted(triggered_scopes):
        for cmd in scope_commands.get(scope, []):
            if cmd not in recommended_commands:
                recommended_commands.append(cmd)

    # Minimal: always cargo check.
    recommended_commands.append("cargo check --workspace --tests --all-features")

    # Docs commands.
    if docs_only or knowledge_base_changed:
        for cmd in docs_commands:
            if cmd not in recommended_commands:
                recommended_commands.append(cmd)

    # Code scope.
    if "code" in recommended_scopes:
        for cmd in code_commands:
            if cmd not in recommended_commands:
                recommended_commands.append(cmd)
        targeted_modules = sorted(affected_modules)
        if broad_impact or not targeted_modules:
            recommended_commands.append("cargo nextest run --workspace")
        else:
            package_args = " ".join(f"-p {m}" for m in targeted_modules)
            recommended_commands.append(f"cargo nextest run {package_args}")

    return {
        "analysis_source": analysis_source,
        "changed_files": changed_files,
        "changed_file_count": len(changed_files),
        "affected_areas": {
            "proto": any(
                s == "proto" for s in triggered_scopes
            ),
            "proof": any(
                s == "proof" for s in triggered_scopes
            ),
            "crates": crates,
            "docs_or_prose": [
                path for path in changed_files if is_prose_or_docs(path)
            ],
        },
        "affected_modules": sorted(affected_modules),
        "risk_flags": risk_flags,
        "docs_only": docs_only,
        "recommended_scopes": recommended_scopes,
        "recommended_commands": recommended_commands,
        "broad_impact": broad_impact,
    }


def print_text(analysis: dict[str, Any]) -> None:
    print(f"analysis_source: {analysis['analysis_source']}")
    print(f"changed_file_count: {analysis['changed_file_count']}")
    print(f"docs_only: {analysis['docs_only']}")
    print(f"affected_modules: {', '.join(analysis['affected_modules']) or 'none'}")
    print(f"risk_flags: {', '.join(analysis['risk_flags']) or 'none'}")
    print(f"recommended_scopes: {', '.join(analysis['recommended_scopes'])}")
    print("recommended_commands:")
    for command in analysis["recommended_commands"]:
        print(f"  - {command}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Analyze local git changes and recommend verification scopes.",
    )
    parser.add_argument(
        "--format",
        choices=("json", "text"),
        default="json",
        help="Output format.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        config = load_config()
        changed_files, analysis_source = list_changed_files()
        analysis = parse_analysis(changed_files, analysis_source, config)
    except RuntimeError as error:
        print(str(error), file=sys.stderr)
        return 1

    if args.format == "text":
        print_text(analysis)
        return 0

    print(json.dumps(analysis, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

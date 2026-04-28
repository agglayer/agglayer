#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any


PROSE_EXTENSIONS = {".md", ".adoc", ".rst", ".txt"}
PROTO_CONFIG_FILES = {
    "buf.yaml",
    "buf.rust.gen.yaml",
    "buf.storage.gen.yaml",
}
RUNTIME_MARKER_FILES = {
    "Cargo.toml",
    "Cargo.lock",
    "Makefile.toml",
    "rust-toolchain",
    "rust-toolchain.toml",
    *PROTO_CONFIG_FILES,
}
CORE_BROAD_CRATES = {
    "agglayer-types",
    "agglayer-storage",
    "agglayer-config",
    "agglayer-rpc",
    "agglayer-jsonrpc-api",
    "agglayer-grpc-api",
    "agglayer-grpc-types",
    "agglayer-node",
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


def parse_analysis(changed_files: list[str], analysis_source: str) -> dict[str, Any]:
    crates = unique(
        [name for path in changed_files if (name := crate_name(path)) is not None]
    )
    knowledge_base_changed = any(path.startswith("docs/knowledge-base/") for path in changed_files)
    proto_changed = any(
        path.startswith("proto/") or Path(path).name in PROTO_CONFIG_FILES
        for path in changed_files
    )
    proof_changed = any(
        path.startswith("crates/pessimistic-proof") for path in changed_files
    )

    docs_only = bool(changed_files) and all(is_prose_or_docs(path) for path in changed_files)
    runtime_behavior_may_change = any(
        path.startswith("crates/")
        or path.startswith("proto/")
        or path.startswith("tests/")
        or Path(path).name in RUNTIME_MARKER_FILES
        for path in changed_files
    )

    affected_crates = set(crates)
    if proto_changed:
        affected_crates.update(
            {
                "agglayer-grpc-api",
                "agglayer-grpc-client",
                "agglayer-grpc-server",
                "agglayer-grpc-types",
                "agglayer-storage",
            }
        )

    risk_flags: list[str] = []
    if proof_changed:
        risk_flags.append("proof pipeline changes")
    if proto_changed:
        risk_flags.append("protobuf schema changes")
    if any(
        path.startswith("crates/agglayer-storage/") or path.startswith("proto/agglayer/storage/")
        for path in changed_files
    ):
        risk_flags.append("storage schema/migration changes")
    if any(
        path.startswith("crates/agglayer-settlement-service/")
        or path.startswith("crates/agglayer-signer/")
        or path.startswith("crates/agglayer-contracts/")
        for path in changed_files
    ):
        risk_flags.append("settlement/signer/contract changes")
    if any(path.startswith("crates/agglayer-config/") for path in changed_files):
        risk_flags.append("configuration schema changes")

    recommended_scopes = ["minimal"]
    if runtime_behavior_may_change and not docs_only:
        recommended_scopes.append("code")
    if proof_changed:
        recommended_scopes.append("proof")
    if proto_changed:
        recommended_scopes.append("proto")

    broad_impact = bool(
        proto_changed
        or len(affected_crates) >= 4
        or any(crate in CORE_BROAD_CRATES for crate in affected_crates)
    )

    recommended_commands = []
    if "proto" in recommended_scopes:
        recommended_commands.append("cargo make generate-proto")

    recommended_commands.append("cargo check --workspace --tests --all-features")
    if docs_only or knowledge_base_changed:
        recommended_commands.append("mdbook build docs/knowledge-base/")

    if "code" in recommended_scopes:
        recommended_commands.append("cargo make ci-all")
        targeted_crates = sorted(affected_crates)
        if broad_impact or not targeted_crates:
            recommended_commands.append("cargo nextest run --workspace")
        else:
            package_args = " ".join(f"-p {crate}" for crate in targeted_crates)
            recommended_commands.append(f"cargo nextest run {package_args}")

    if "proof" in recommended_scopes:
        recommended_commands.append("cargo make pp-check-vkey-change")

    return {
        "analysis_source": analysis_source,
        "changed_files": changed_files,
        "changed_file_count": len(changed_files),
        "affected_areas": {
            "proto": proto_changed,
            "proof": proof_changed,
            "crates": crates,
            "docs_or_prose": [path for path in changed_files if is_prose_or_docs(path)],
        },
        "affected_crates": sorted(affected_crates),
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
    print(f"affected_crates: {', '.join(analysis['affected_crates']) or 'none'}")
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
        changed_files, analysis_source = list_changed_files()
        analysis = parse_analysis(changed_files, analysis_source)
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

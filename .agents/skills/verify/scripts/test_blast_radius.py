#!/usr/bin/env python3
"""Tests for blast_radius.py."""

from __future__ import annotations

import pytest

from unittest.mock import patch

from blast_radius import is_prose_or_docs, split_nonempty_lines, crate_name, parse_analysis, match_pattern, load_config, DEFAULT_CONFIG


class TestIsProseOrDocs:
    def test_docs_directory(self):
        assert is_prose_or_docs("docs/architecture.md") is True

    def test_docs_subdirectory(self):
        assert is_prose_or_docs("docs/knowledge-base/src/SUMMARY.md") is True

    def test_markdown_file(self):
        assert is_prose_or_docs("README.md") is True

    def test_adoc_file(self):
        assert is_prose_or_docs("notes.adoc") is True

    def test_rst_file(self):
        assert is_prose_or_docs("changelog.rst") is True

    def test_txt_file(self):
        assert is_prose_or_docs("notes.txt") is True

    def test_contributing(self):
        assert is_prose_or_docs("CONTRIBUTING.md") is True

    def test_license(self):
        assert is_prose_or_docs("LICENSE") is True

    def test_changelog(self):
        assert is_prose_or_docs("CHANGELOG.md") is True

    def test_agents_md(self):
        assert is_prose_or_docs("AGENTS.md") is True

    def test_rust_source(self):
        assert is_prose_or_docs("crates/foo/src/lib.rs") is False

    def test_cargo_toml(self):
        assert is_prose_or_docs("Cargo.toml") is False

    def test_proto_file(self):
        assert is_prose_or_docs("proto/types.proto") is False


class TestSplitNonemptyLines:
    def test_empty_string(self):
        assert split_nonempty_lines("") == []

    def test_single_line(self):
        assert split_nonempty_lines("foo") == ["foo"]

    def test_multiple_lines(self):
        assert split_nonempty_lines("a\nb\nc") == ["a", "b", "c"]

    def test_blank_lines_filtered(self):
        assert split_nonempty_lines("a\n\nb\n") == ["a", "b"]


class TestCrateName:
    def test_crate_path(self):
        assert crate_name("crates/agglayer-types/src/lib.rs") == "agglayer-types"

    def test_crate_root_file(self):
        assert crate_name("crates/foo/Cargo.toml") == "foo"

    def test_not_in_crates(self):
        assert crate_name("proto/types.proto") is None

    def test_root_file(self):
        assert crate_name("Cargo.toml") is None

    def test_crates_dir_only(self):
        # "crates/".split("/", 2) == ["crates", ""] -> returns "" not None
        assert crate_name("crates/") == ""


class TestParseAnalysisOutputShape:
    """Verify the output dict has all expected keys."""

    def test_empty_input(self):
        result = parse_analysis([], "none", DEFAULT_CONFIG)
        assert result["analysis_source"] == "none"
        assert result["changed_files"] == []
        assert result["changed_file_count"] == 0
        assert result["docs_only"] is False
        assert result["broad_impact"] is False
        assert "affected_modules" in result
        assert "risk_flags" in result
        assert "recommended_scopes" in result
        assert "recommended_commands" in result

    def test_docs_only(self):
        result = parse_analysis(
            ["docs/foo.md", "README.md"], "working-tree", DEFAULT_CONFIG
        )
        assert result["docs_only"] is True
        assert "minimal" in result["recommended_scopes"]

    def test_crate_change(self):
        config = dict(DEFAULT_CONFIG)
        config["core_crates"] = ["agglayer-types"]
        result = parse_analysis(
            ["crates/agglayer-types/src/lib.rs"], "main...HEAD", config
        )
        assert result["docs_only"] is False
        assert "agglayer-types" in result["affected_modules"]
        assert "code" in result["recommended_scopes"]


class TestMatchPattern:
    def test_exact_match(self):
        assert match_pattern("buf.yaml", "buf.yaml") is True

    def test_prefix_match_directory(self):
        assert match_pattern("proto/types.proto", "proto/") is True

    def test_prefix_no_match(self):
        assert match_pattern("src/proto/types.proto", "proto/") is False

    def test_glob_suffix(self):
        assert match_pattern(
            "crates/pessimistic-proof/src/lib.rs", "crates/pessimistic-proof"
        ) is True

    def test_glob_suffix_related_crate(self):
        assert match_pattern(
            "crates/pessimistic-proof-test-suite/tests/foo.rs",
            "crates/pessimistic-proof",
        ) is True

    def test_no_match(self):
        assert match_pattern("crates/agglayer-types/lib.rs", "proto/") is False

    def test_empty_pattern(self):
        assert match_pattern("anything", "") is True

    def test_empty_path(self):
        assert match_pattern("", "proto/") is False


class TestLoadConfig:
    def test_returns_default_when_no_file(self, tmp_path):
        with patch("blast_radius.run_git", return_value=str(tmp_path)):
            config = load_config()
        assert config == DEFAULT_CONFIG

    def test_loads_yaml_config(self, tmp_path):
        config_content = (
            "core_crates:\n"
            "  - my-crate\n"
            "risk_areas: []\n"
            "docs_commands: []\n"
            "code_commands: []\n"
        )
        (tmp_path / ".blast-radius.yaml").write_text(config_content)
        with patch("blast_radius.run_git", return_value=str(tmp_path)):
            config = load_config()
        assert config["core_crates"] == ["my-crate"]
        assert config["risk_areas"] == []

    def test_missing_keys_get_defaults(self, tmp_path):
        config_content = "core_crates:\n  - foo\n"
        (tmp_path / ".blast-radius.yaml").write_text(config_content)
        with patch("blast_radius.run_git", return_value=str(tmp_path)):
            config = load_config()
        assert config["core_crates"] == ["foo"]
        assert config["risk_areas"] == []
        assert config["docs_commands"] == []
        assert config["code_commands"] == []

    def test_empty_yaml_returns_defaults(self, tmp_path):
        (tmp_path / ".blast-radius.yaml").write_text("")
        with patch("blast_radius.run_git", return_value=str(tmp_path)):
            config = load_config()
        assert config == DEFAULT_CONFIG


AGGLAYER_CONFIG: dict = {
    "core_crates": [
        "agglayer-types",
        "agglayer-storage",
        "agglayer-config",
        "agglayer-rpc",
        "agglayer-jsonrpc-api",
        "agglayer-grpc-api",
        "agglayer-grpc-types",
        "agglayer-node",
    ],
    "risk_areas": [
        {
            "name": "proof pipeline changes",
            "patterns": ["crates/pessimistic-proof"],
            "scope": "proof",
            "commands": ["cargo make pp-check-vkey-change"],
        },
        {
            "name": "protobuf schema changes",
            "patterns": ["proto/", "buf.yaml", "buf.rust.gen.yaml", "buf.storage.gen.yaml"],
            "scope": "proto",
            "propagates_to": [
                "agglayer-grpc-api",
                "agglayer-grpc-client",
                "agglayer-grpc-server",
                "agglayer-grpc-types",
                "agglayer-storage",
            ],
            "commands": ["cargo make generate-proto"],
        },
        {
            "name": "storage schema/migration changes",
            "patterns": ["crates/agglayer-storage/", "proto/agglayer/storage/"],
        },
        {
            "name": "settlement/signer/contract changes",
            "patterns": [
                "crates/agglayer-settlement-service/",
                "crates/agglayer-signer/",
                "crates/agglayer-contracts/",
            ],
        },
        {
            "name": "configuration schema changes",
            "patterns": ["crates/agglayer-config/"],
        },
    ],
    "docs_commands": ["mdbook build docs/knowledge-base/"],
    "code_commands": ["cargo make ci-all"],
}


class TestParseAnalysisConfigDriven:
    """Test parse_analysis with explicit config (new signature)."""

    def test_empty_with_default_config(self):
        result = parse_analysis([], "none", DEFAULT_CONFIG)
        assert result["changed_file_count"] == 0
        assert result["docs_only"] is False
        assert result["risk_flags"] == []
        assert result["recommended_scopes"] == ["minimal"]
        assert result["broad_impact"] is False

    def test_docs_only_with_config(self):
        result = parse_analysis(
            ["docs/foo.md", "README.md"], "working-tree", AGGLAYER_CONFIG
        )
        assert result["docs_only"] is True
        assert "mdbook build docs/knowledge-base/" in result["recommended_commands"]

    def test_proof_risk_flagged(self):
        result = parse_analysis(
            ["crates/pessimistic-proof/src/lib.rs"], "main...HEAD", AGGLAYER_CONFIG
        )
        assert "proof pipeline changes" in result["risk_flags"]
        assert "proof" in result["recommended_scopes"]
        assert "cargo make pp-check-vkey-change" in result["recommended_commands"]

    def test_proto_risk_with_propagation(self):
        result = parse_analysis(
            ["proto/agglayer/types.proto"], "main...HEAD", AGGLAYER_CONFIG
        )
        assert "protobuf schema changes" in result["risk_flags"]
        assert "proto" in result["recommended_scopes"]
        assert "agglayer-grpc-api" in result["affected_modules"]
        assert "agglayer-storage" in result["affected_modules"]
        assert "cargo make generate-proto" in result["recommended_commands"]

    def test_core_crate_triggers_broad_impact(self):
        result = parse_analysis(
            ["crates/agglayer-types/src/lib.rs"], "main...HEAD", AGGLAYER_CONFIG
        )
        assert result["broad_impact"] is True

    def test_non_core_crate_no_broad_impact(self):
        result = parse_analysis(
            ["crates/agglayer-prover/src/lib.rs"], "main...HEAD", AGGLAYER_CONFIG
        )
        assert result["broad_impact"] is False

    def test_many_crates_trigger_broad_impact(self):
        files = [
            "crates/a/src/lib.rs",
            "crates/b/src/lib.rs",
            "crates/c/src/lib.rs",
            "crates/d/src/lib.rs",
        ]
        result = parse_analysis(files, "main...HEAD", AGGLAYER_CONFIG)
        assert result["broad_impact"] is True

    def test_storage_risk_flagged(self):
        result = parse_analysis(
            ["crates/agglayer-storage/migrations/001.sql"], "main...HEAD", AGGLAYER_CONFIG
        )
        assert "storage schema/migration changes" in result["risk_flags"]

    def test_settlement_risk_flagged(self):
        result = parse_analysis(
            ["crates/agglayer-signer/src/lib.rs"], "main...HEAD", AGGLAYER_CONFIG
        )
        assert "settlement/signer/contract changes" in result["risk_flags"]

    def test_config_risk_flagged(self):
        result = parse_analysis(
            ["crates/agglayer-config/src/lib.rs"], "main...HEAD", AGGLAYER_CONFIG
        )
        assert "configuration schema changes" in result["risk_flags"]

    def test_code_commands_included(self):
        result = parse_analysis(
            ["crates/agglayer-types/src/lib.rs"], "main...HEAD", AGGLAYER_CONFIG
        )
        assert "cargo make ci-all" in result["recommended_commands"]

    def test_no_config_still_detects_crates(self):
        result = parse_analysis(
            ["crates/my-crate/src/lib.rs"], "main...HEAD", DEFAULT_CONFIG
        )
        assert "my-crate" in result["affected_modules"]
        assert "code" in result["recommended_scopes"]

    def test_knowledge_base_triggers_docs_commands(self):
        result = parse_analysis(
            ["docs/knowledge-base/src/chapter.md", "crates/foo/src/lib.rs"],
            "main...HEAD",
            AGGLAYER_CONFIG,
        )
        assert "mdbook build docs/knowledge-base/" in result["recommended_commands"]

    def test_workspace_nextest_on_broad_impact(self):
        result = parse_analysis(
            ["crates/agglayer-types/src/lib.rs"], "main...HEAD", AGGLAYER_CONFIG
        )
        assert "cargo nextest run --workspace" in result["recommended_commands"]

    def test_targeted_nextest_on_narrow_change(self):
        result = parse_analysis(
            ["crates/agglayer-prover/src/lib.rs"], "main...HEAD", AGGLAYER_CONFIG
        )
        targeted = [
            c for c in result["recommended_commands"] if "nextest" in c
        ]
        assert len(targeted) == 1
        assert "-p agglayer-prover" in targeted[0]

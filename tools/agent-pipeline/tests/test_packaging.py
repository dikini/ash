"""Tests for packaged deployment assets."""

from __future__ import annotations

from pathlib import Path


TOOL_DIR = Path(__file__).resolve().parents[1]


def _read(relative_path: str) -> str:
    """Read a packaged deployment asset as text."""
    return (TOOL_DIR / relative_path).read_text()


def test_packaging_assets_do_not_hardcode_clone_location() -> None:
    """RED: Install helpers should be portable across clone locations."""
    install_script = _read("install.sh")
    service_template = _read("agent-pipeline.service")
    vila_script = _read("vila-integration.sh")

    forbidden_fragments = ["$HOME/Projects/ash", "~/Projects/ash", "%h/Projects/ash"]
    for fragment in forbidden_fragments:
        assert fragment not in install_script
        assert fragment not in service_template
        assert fragment not in vila_script


def test_service_template_uses_explicit_pipeline_env_configuration() -> None:
    """RED: Packaged service should set explicit workspace and state env values."""
    service_template = _read("agent-pipeline.service")

    assert "Environment=AGENT_PIPELINE_WORKSPACE_ROOT=" in service_template
    assert "Environment=AGENT_PIPELINE_BASE_DIR=" in service_template
    assert "Environment=PATH=%h/.cargo/bin:%h/.local/bin:/usr/local/bin:/usr/bin:/bin" in service_template


def test_install_and_vila_scripts_derive_paths_from_their_own_location() -> None:
    """RED: Helper scripts should discover the clone root dynamically."""
    install_script = _read("install.sh")
    vila_script = _read("vila-integration.sh")

    assert "SCRIPT_DIR=" in install_script
    assert "REPO_ROOT=" in install_script
    assert "SCRIPT_DIR=" in vila_script
    assert "REPO_ROOT=" in vila_script

"""Shared pytest wiring for the espectacular contract tests.

The cargo-binary helper lives here (task 1.4 TIDY): every contract test
needs the freshly built specodelic binary, and building `cargo metadata`
once per session keeps the property test's wall-clock sane.
"""

import json
import os
import subprocess

import pytest


@pytest.fixture(scope="session")
def spk_binary() -> str:
    """The freshly built specodelic binary (target/debug)."""
    root = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        capture_output=True,
        text=True,
        check=True,
    )
    target = json.loads(root.stdout)["target_directory"]
    binary = os.path.join(target, "debug", "specodelic")
    assert os.path.exists(binary), f"binary not built: {binary}"
    return binary

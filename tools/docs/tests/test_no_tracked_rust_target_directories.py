#!/usr/bin/env python3
"""Regression contract: Cargo target directories are never tracked."""
from __future__ import annotations

import subprocess
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[3]


class NoTrackedRustTargetDirectoriesTests(unittest.TestCase):
    """Keep generated Cargo output outside the repository index."""

    def git(self, *arguments: str) -> subprocess.CompletedProcess[bytes]:
        """Run Git against the checked-out repository."""
        return subprocess.run(
            ["git", *arguments],
            check=False,
            capture_output=True,
            cwd=REPOSITORY_ROOT,
        )

    def test_no_tracked_path_is_inside_a_target_directory(self) -> None:
        """The index contains no Cargo output under a directory named ``target``."""
        result = self.git("ls-files", "-z")
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors="replace"))

        tracked_paths = result.stdout.decode(errors="surrogateescape").split("\0")
        target_paths = [
            path
            for path in tracked_paths
            if path and "/target/" in f"/{path}"
        ]

        self.assertFalse(
            target_paths,
            "tracked Cargo target paths must be removed from the index:\n"
            + "\n".join(target_paths),
        )

    def test_target_ignore_rule_covers_root_and_nested_cargo_output(self) -> None:
        """A global rule ignores Cargo output at every repository depth."""
        for path in (
            "target/.rustc_info.json",
            "crates/example/target/.rustc_info.json",
        ):
            with self.subTest(path=path):
                result = self.git("check-ignore", "-q", "--", path)
                self.assertEqual(
                    result.returncode,
                    0,
                    f"expected a global target/ ignore rule for {path}; "
                    f"stderr: {result.stderr.decode(errors='replace')}",
                )


if __name__ == "__main__":
    unittest.main()

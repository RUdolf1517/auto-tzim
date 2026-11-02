#!/usr/bin/env python3
"""Exercise the actual checker on isolated fixture manifests."""
import pathlib
import shutil
import subprocess
import sys
import tempfile

checker = pathlib.Path(__file__).with_name("check-pins.py")
with tempfile.TemporaryDirectory() as temp:
    root = pathlib.Path(temp)
    (root / "scripts").mkdir()
    shutil.copy(checker, root / "scripts" / checker.name)
    for requirement, expected in (("*", 1), ("=1.2.3", 0)):
        (root / "Cargo.toml").write_text(f'[dependencies]\nfixture = "{requirement}"\n')
        result = subprocess.run(
            [sys.executable, str(root / "scripts" / checker.name)],
            capture_output=True,
            text=True,
        )
        if result.returncode != expected:
            raise AssertionError(
                f"{requirement!r}: expected {expected}, got {result.returncode}\n"
                f"stdout: {result.stdout}\nstderr: {result.stderr}"
            )
print("Pin checker self-test passed")

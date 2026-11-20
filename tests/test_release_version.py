import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "check-release-version.py"


class ReleaseVersionTest(unittest.TestCase):
    def check(self, tag, manifest):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "Cargo.toml"
            path.write_text(manifest, encoding="utf-8")
            return subprocess.run(
                [sys.executable, str(SCRIPT), str(path), tag],
                capture_output=True,
                text=True,
                env=os.environ.copy(),
                check=False,
            )

    def test_matching_tag_is_allowed(self):
        self.assertEqual(self.check("v1.2.3", '[package]\nversion = "1.2.3"\n').returncode, 0)

    def test_mismatched_tag_is_rejected(self):
        self.assertNotEqual(self.check("v9.0.0", '[package]\nversion = "1.2.3"\n').returncode, 0)

    def test_invalid_manifest_is_rejected(self):
        self.assertNotEqual(self.check("v1.2.3", "not a manifest").returncode, 0)


if __name__ == "__main__":
    unittest.main()

import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
ENGINE = ROOT / 'scripts/token_sync.py'
if Path(__file__).parent.name == "checks":
    ROOT = Path(__file__).resolve().parents[2]
    ENGINE = ROOT / 'rendering/token_sync.py'


class TokenSyncTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.site = self.directory / 'ash-site'
        self.doc = self.directory / 'ash-doc'
        self.site_token = self.site / 'public/tokens.css'
        self.doc_token = self.doc / 'rendering/styles/ash-tokens.css'
        for token in [self.site_token, self.doc_token]:
            token.parent.mkdir(parents=True)
            token.write_text(':root { --ink: black; }\n')
            (token.parents[1] if token == self.site_token else token.parents[2]).joinpath('.design-tokens-base.css').write_text(token.read_text())
        (self.doc / 'rendering/styles/html.scss').touch()
        self.env = {k: v for k, v in os.environ.items() if k not in ['ASH_SITE_CHECKOUT', 'ASH_DOC_CHECKOUT', 'ASH_TOKENS_STANDALONE']}

    def run_sync(self, root=None, *args, env=None):
        return subprocess.run(['python3', str(ENGINE), '--root', str(root or self.site), *args],
                              capture_output=True, text=True, env=env or self.env)

    def snapshot(self):
        return {str(p): p.read_bytes() for p in self.directory.rglob('*') if p.is_file()}

    def test_syncs_site_edits_and_updates_both_baselines(self):
        self.site_token.write_text(':root { --ink: green; }\n')
        result = self.run_sync()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.site_token.read_bytes(), self.doc_token.read_bytes())
        for root in [self.site, self.doc]:
            self.assertEqual((root / '.design-tokens-base.css').read_bytes(), self.site_token.read_bytes())

    def test_syncs_documentation_edits_from_documentation_entry_point(self):
        self.doc_token.write_text(':root { --ink: brown; }\n')
        result = self.run_sync(self.doc)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.site_token.read_bytes(), self.doc_token.read_bytes())

    def test_check_reports_drift_without_writing(self):
        self.doc_token.write_text(':root { --ink: brown; }\n')
        before = self.snapshot()
        result = self.run_sync(None, '--check')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('drift', result.stderr.lower())
        self.assertEqual(before, self.snapshot())

    def test_conflicting_edits_preserve_all_files_and_show_diff(self):
        self.site_token.write_text(':root { --ink: green; }\n')
        self.doc_token.write_text(':root { --ink: brown; }\n')
        before = self.snapshot()
        result = self.run_sync()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('conflict', result.stderr.lower())
        self.assertIn('--ink: green', result.stderr)
        self.assertIn('--ink: brown', result.stderr)
        self.assertEqual(before, self.snapshot())

    def test_identical_edits_refresh_baselines(self):
        for p in [self.site_token, self.doc_token]:
            p.write_text(':root { --ink: green; }\n')
        self.assertEqual(self.run_sync().returncode, 0)
        self.assertEqual(self.run_sync(None, '--check').returncode, 0)

    def test_different_baselines_fail_without_writing(self):
        self.site_token.write_text(':root { --ink: green; }\n')
        (self.doc / '.design-tokens-base.css').write_text('different baseline')
        before = self.snapshot()
        result = self.run_sync()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('baseline', result.stderr)
        self.assertEqual(before, self.snapshot())

    def test_missing_automatic_peer_builds_standalone_without_advancing_baseline(self):
        self.doc.rename(self.directory / 'elsewhere')
        self.site_token.write_text(':root { --ink: green; }\n')
        before = self.snapshot()
        result = self.run_sync()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('standalone', result.stdout.lower())
        self.assertEqual(before, self.snapshot())

    def test_invalid_explicit_peer_fails(self):
        result = self.run_sync(None, '--peer', str(self.directory / 'missing'))
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('peer', result.stderr.lower())

    def test_explicit_peer_override(self):
        elsewhere = self.directory / 'elsewhere'
        self.doc.rename(elsewhere)
        self.site_token.write_text(':root { --ink: green; }\n')
        env = dict(self.env, ASH_DOC_CHECKOUT=str(elsewhere))
        self.assertEqual(self.run_sync(env=env).returncode, 0)
        self.assertEqual((elsewhere / 'rendering/styles/ash-tokens.css').read_bytes(), self.site_token.read_bytes())

    def test_standalone_override_ignores_existing_peer(self):
        self.doc_token.write_text('peer edit')
        before = self.snapshot()
        result = self.run_sync(env=dict(self.env, ASH_TOKENS_STANDALONE='1'))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(before, self.snapshot())

    def test_missing_peer_token_is_an_error(self):
        self.doc_token.unlink()
        self.assertNotEqual(self.run_sync().returncode, 0)

    def test_worktree_discovers_peer_beside_primary_checkout(self):
        subprocess.run(['git', 'init', '-q', str(self.site)], check=True)
        subprocess.run(['git', '-C', str(self.site), 'add', '.'], check=True)
        subprocess.run(['git', '-C', str(self.site), '-c', 'user.name=Test', '-c', 'user.email=test@example.test', 'commit', '-qm', 'baseline'], check=True)
        worktree = self.site / '.worktrees/task'
        subprocess.run(['git', '-C', str(self.site), 'worktree', 'add', '-qb', 'task', str(worktree)], check=True)
        (worktree / 'public/tokens.css').write_text(':root { --ink: green; }\n')
        result = self.run_sync(worktree)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.doc_token.read_bytes(), (worktree / 'public/tokens.css').read_bytes())


if __name__ == '__main__':
    unittest.main()

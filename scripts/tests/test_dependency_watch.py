import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("watch", Path(__file__).parents[1] / "check-dependencies.py")
watch = importlib.util.module_from_spec(spec)
spec.loader.exec_module(watch)


class DependencyWatch(unittest.TestCase):
    def fixture(self, url):
        if "channel-rust" in url:
            return '[pkg.rust]\nversion = "1.100.0 (fixture)"'
        if "RELEASES.md" in url:
            return 'Version 1.100.0\nCompiler notes\nVersion 1.99.0\nOld notes'
        if "crates.io" in url:
            return json.dumps({"crate": {"max_stable_version": "0.9.0"}})
        if "tursodatabase" in url:
            return json.dumps([{"tag_name": "v0.9.0", "body": "Storage changes"}])
        if "/compare/" in url:
            return json.dumps({"status": "ahead", "commits": [{"commit": {"message": "Change UI API"}}], "files": [{"filename": "changelog/unreleased/fix.md"}]})
        if url.endswith('/anomalyco/rune'):
            return json.dumps({"default_branch": "dev"})
        self.assertTrue(url.endswith('/commits/dev'), url)
        return json.dumps({"sha": "b" * 40})

    def test_versions_and_source_notes_are_collected_without_editing_pins(self):
        pins = (watch.ROOT / "folio-update.json").read_bytes()
        report = watch.collect(self.fixture)
        self.assertTrue(all(r['status'] == 'review required' for r in report['dependencies']))
        self.assertEqual(report['dependencies'][2]['changed_files'], ['changelog/unreleased/fix.md'])
        self.assertIn('Storage changes', report['dependencies'][1]['upstream_changes'])
        self.assertEqual((watch.ROOT / "folio-update.json").read_bytes(), pins)

    def test_unavailable_source_is_never_reported_current(self):
        def unavailable(url):
            if 'crates.io' in url:
                raise OSError('offline')
            return self.fixture(url)
        report = watch.collect(unavailable)
        self.assertEqual(report['dependencies'][1]['status'], 'check failed')
        self.assertIn('offline', watch.markdown(report))

    def test_changelog_failure_remains_visible_even_with_known_version(self):
        def unavailable(url):
            if 'RELEASES.md' in url:
                raise OSError('notes unavailable')
            return self.fixture(url)
        report = watch.collect(unavailable)
        self.assertEqual(report['dependencies'][0]['status'], 'changelog check failed')
        self.assertIn('notes unavailable', watch.markdown(report))


if __name__ == '__main__':
    unittest.main()

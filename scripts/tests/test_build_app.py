import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("build_app", Path(__file__).parents[1] / "build-app.py")
build_app = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build_app)


class RuneProvenance(unittest.TestCase):
    def setUp(self):
        self.pin = 'a' * 40
        self.repository = 'https://github.com/anomalyco/rune.git'
        source = {'git': self.repository, 'rev': self.pin}
        self.manifest = {'dependencies': {'rune': dict(source), 'rune-kit': dict(source)},
                         'patch': {'crates-io': {name: dict(source) for name in build_app.RUNE_PATCHES}}}
        self.metadata = {'packages': [
            {'name': name, 'source': f'git+{self.repository}?rev={self.pin}#{self.pin}',
             'manifest_path': f'/checkout/rune/crates/{name}/Cargo.toml'}
            for name in ['rune', 'rune-kit', *sorted(build_app.RUNE_PATCHES)]
        ]}

    def test_packaging_resolves_cargos_exact_checkout_without_a_sibling_repository(self):
        self.assertEqual(build_app.rune_checkout(self.metadata, self.manifest, self.pin), Path('/checkout/rune'))

    def test_mixed_vendor_revision_is_refused_even_if_metadata_claims_the_pin(self):
        self.manifest['patch']['crates-io']['winit']['rev'] = 'b' * 40
        with self.assertRaisesRegex(ValueError, 'winit differs'):
            build_app.rune_checkout(self.metadata, self.manifest, self.pin)

    def test_missing_new_vendor_patches_are_refused_even_if_the_app_can_compile(self):
        for name in ['naga', 'accesskit_atspi_common']:
            with self.subTest(name=name):
                manifest = copy.deepcopy(self.manifest)
                del manifest['patch']['crates-io'][name]
                with self.assertRaisesRegex(ValueError, f'Missing Rune vendor patches: {name}'):
                    build_app.rune_checkout(self.metadata, manifest, self.pin)

    def test_registry_path_and_other_resolved_revisions_are_refused(self):
        for source in [None, 'registry+https://example.org/index',
                       f'git+{self.repository}?rev={self.pin}#{"b" * 40}']:
            with self.subTest(source=source):
                metadata = copy.deepcopy(self.metadata)
                metadata['packages'][0]['source'] = source
                with self.assertRaisesRegex(ValueError, 'rune did not resolve'):
                    build_app.rune_checkout(metadata, self.manifest, self.pin)

    def test_short_pins_and_unpinned_dependency_forms_are_refused(self):
        with self.assertRaisesRegex(ValueError, 'full reviewed Git revision'):
            build_app.rune_checkout(self.metadata, self.manifest, self.pin[:8])
        self.manifest['dependencies']['rune'] = '0.0.1'
        with self.assertRaisesRegex(ValueError, 'rune differs'):
            build_app.rune_checkout(self.metadata, self.manifest, self.pin)

    def test_transitive_rune_crates_cannot_be_overridden_with_another_source(self):
        self.metadata['packages'].append({'name': 'rune-core', 'source': None,
                                          'manifest_path': '/override/rune-core/Cargo.toml'})
        with self.assertRaisesRegex(ValueError, 'rune-core did not resolve'):
            build_app.rune_checkout(self.metadata, self.manifest, self.pin)


if __name__ == '__main__':
    unittest.main()

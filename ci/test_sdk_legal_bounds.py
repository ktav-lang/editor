"""Проверить measured SDK document и неизменный aggregate bound."""

import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('sdk_legal', Path(__file__).resolve().parents[1] / 'scripts/collect_lsp_sdk_legal.py')
SDK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SDK)


class BoundTests(unittest.TestCase):
    def test_observed_complete_third_party_notice_fits(self):
        SDK.check_bounds(6167634, 204298, 'VS-0-Licenses/1033/ThirdPartyNotices.txt')

    def test_exact_document_bound(self):
        SDK.check_bounds(8 * 1024 * 1024, 0, 'fixture')

    def test_larger_document_rejected(self):
        with self.assertRaisesRegex(ValueError, 'document bound'):
            SDK.check_bounds(8 * 1024 * 1024 + 1, 0, 'fixture')

    def test_aggregate_limit_is_still_32_mib(self):
        self.assertEqual(SDK.MAX_TOTAL_BYTES, 32 * 1024 * 1024)
        SDK.check_bounds(8 * 1024 * 1024, 24 * 1024 * 1024, 'fixture')
        with self.assertRaisesRegex(ValueError, 'document bound'):
            SDK.check_bounds(8 * 1024 * 1024, 24 * 1024 * 1024 + 1, 'fixture')

    def test_negative_size_rejected(self):
        with self.assertRaisesRegex(ValueError, 'document bound'):
            SDK.check_bounds(-1, 0, 'fixture')


if __name__ == '__main__':
    unittest.main()

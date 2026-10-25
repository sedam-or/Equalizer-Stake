"""Tests for the migration ownership map used by scripts/erd.py."""

import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("ritma_erd", ROOT / "scripts/erd.py")
erd = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(erd)


class MigrationOwnershipTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tables, _ = erd.load_spec()
        cls.table_owners, cls.column_owners = erd.migration_ownership(
            (ROOT / "spec/29-erd.md").read_text(), set(cls.tables)
        )

    def test_every_ddl_table_has_a_migration_owner(self):
        self.assertEqual(set(self.tables), set(self.table_owners))

    def test_ics_secret_version_is_owned_by_later_migration(self):
        self.assertEqual(self.table_owners["persons"], "0002_core_tenancy.sql")
        self.assertEqual(
            self.column_owners[("persons", "ics_secret_version")],
            "0012_ics.sql",
        )


if __name__ == "__main__":
    unittest.main()

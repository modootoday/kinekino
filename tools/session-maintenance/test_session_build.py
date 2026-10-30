import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("maintenance", Path(__file__).with_name("session_build.py"))
maintenance = importlib.util.module_from_spec(spec)
spec.loader.exec_module(maintenance)
TEST_ROOT = maintenance.WORKSPACE / "runtime/build-cache/tmp/maintenance-tests"
TEST_ROOT.mkdir(parents=True, exist_ok=True)


class ScratchTests(unittest.TestCase):
    def test_long_paths_and_partial_cleanup_recover_from_durable_success(self):
        with tempfile.TemporaryDirectory(prefix="maintenance-long-", dir=TEST_ROOT) as tmp:
            scratch = maintenance.Scratch(Path(tmp) / "another-layout")
            run = scratch.create()
            nested = run / ("a" * 70) / ("b" * 70)
            os.makedirs(maintenance.long_path(nested))
            with open(maintenance.long_path(nested / "payload.json"), "w") as file:
                file.write("fixture")
            self.assertGreater(len(str(nested / "payload.json")), 260)
            # Reproduce a previous successful runner whose root markers were
            # removed before Windows failed on a deep path.
            (run / "owner.json").unlink()
            scratch.logs.mkdir(parents=True)
            maintenance.write_new(scratch.logs / "fixture.maintenance.json",
                                  {"schema": "source-build-maintenance/1",
                                   "scratch": str(run), "exit_code": 0, "live_after": []})
            self.assertEqual(scratch.clean(False), [str(run)])
            self.assertEqual(scratch.clean(True), [str(run)])
            self.assertFalse(run.exists())

    def test_only_new_loose_intermediates_are_removed(self):
        with tempfile.TemporaryDirectory(prefix="maintenance-loose-", dir=TEST_ROOT) as tmp:
            deps = Path(tmp) / "unrelated-layout" / "objects"
            deps.mkdir(parents=True)
            (deps / "prior.rcgu.o").write_bytes(b"preserve prior failure")
            (deps / "proof.exe").write_bytes(b"preserve executable")
            before = maintenance.leftover_inventory(deps)
            (deps / "this-run.rcgu.o").write_bytes(b"new temporary")
            self.assertEqual(maintenance.clean_new_leftovers(deps, before), [str(deps / "this-run.rcgu.o")])
            self.assertTrue((deps / "prior.rcgu.o").exists())
            self.assertTrue((deps / "proof.exe").exists())

    def test_process_inventory_has_known_present_control(self):
        if os.name != "nt":
            self.skipTest("Native Windows process API")
        self.assertTrue(any(row.endswith(":" + str(os.getpid()))
                            for row in maintenance.active_builds({"python.exe"})))

    def test_exclusive_build_slot_rejects_second_owner(self):
        if os.name != "nt":
            self.skipTest("Native Windows file lock")
        with tempfile.TemporaryDirectory(prefix="maintenance-slot-", dir=TEST_ROOT) as tmp:
            path = Path(tmp) / "build.lock"
            with maintenance.exclusive(path):
                with self.assertRaises(OSError):
                    with maintenance.exclusive(path):
                        self.fail("Second owner acquired slot")

    def test_only_owned_completed_scratch_is_removed(self):
        with tempfile.TemporaryDirectory(prefix="maintenance-control-", dir=TEST_ROOT) as tmp:
            root = Path(tmp) / "independent-repository"
            scratch = maintenance.Scratch(root)
            success = scratch.create()
            (success / "rebuildable.bin").write_bytes(b"fixture")
            maintenance.write_new(success / "completed.json", {"run": success.name, "exit_code": 0})
            failed = scratch.create()
            (failed / "first-failure.txt").write_text("preserve")
            foreign = scratch.root / ("run-" + "a" * 32)
            foreign.mkdir()
            (foreign / "owner.json").write_text(json.dumps({"schema": "foreign"}))
            proof = root / "runtime/verification/fixed-r250"
            proof.mkdir(parents=True)
            (proof / "proof.exe").write_bytes(b"keep")
            self.assertEqual(scratch.clean(False), [str(success)])
            self.assertTrue(success.exists())
            self.assertEqual(scratch.clean(True), [str(success)])
            self.assertFalse(success.exists())
            self.assertTrue(failed.exists())
            self.assertTrue(foreign.exists())
            self.assertEqual((proof / "proof.exe").read_bytes(), b"keep")
            self.assertEqual(scratch.clean(True), [])

    def test_forged_completion_and_path_name_are_not_selected(self):
        with tempfile.TemporaryDirectory(prefix="maintenance-negative-", dir=TEST_ROOT) as tmp:
            scratch = maintenance.Scratch(Path(tmp))
            run = scratch.create()
            maintenance.write_new(run / "completed.json", {"run": "another", "exit_code": 0})
            self.assertEqual(scratch.clean(False), [])
            (scratch.root / "not-a-managed-run").mkdir()
            self.assertEqual(scratch.clean(True), [])
            self.assertTrue(run.exists())


if __name__ == "__main__":
    unittest.main()

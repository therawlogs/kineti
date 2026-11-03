"""Adversarial local tests. All identities, costs, approvals and writes are synthetic."""
import copy
import io
import json
import multiprocessing
import os
import shutil
from pathlib import Path
import tarfile
import tempfile
import unittest
import uuid
from unittest.mock import patch

from controller import (Audit, Controller, Denied, ProductionAuthority, SyntheticAuthority,
                        archive_bytes, canonical, digest, identity, parse, schema)


NOW = 1_800_000_000
BASE = Path(__file__).resolve().parents[1] / ".kineti" / "production-release-implementation" / "fixtures"
BASE.mkdir(parents=True, exist_ok=True)


def tar_bytes(name="package/file", kind=None, data=b"synthetic payload"):
    out = io.BytesIO()
    with tarfile.open(fileobj=out, mode="w:gz") as archive:
        item = tarfile.TarInfo(name)
        item.size = len(data)
        if kind is not None:
            item.type = kind
            item.linkname = "../../outside"
        archive.addfile(item, io.BytesIO(data))
    return out.getvalue()


def fixture(root):
    root = Path(root)
    (root / "archives").mkdir()
    (root / "audit").mkdir()
    h = "a" * 64
    workflow = {"repository_id": 2, "path": ".github/workflows/controller.yml",
                "revision": "b" * 40, "issuer_id": 3, "environment": "synthetic-review"}
    declarations = {"native": {"issuer_id": 3, "runner": "synthetic-linux-x64", "assertions": 52,
                               "argv": ["cargo", "test", "", "--locked"], "executable_sha256": h}}
    runners = {target: "synthetic-" + target for target in
               ("linux-x64", "linux-arm64", "darwin-x64", "darwin-arm64")}
    policy = {"schema": 1, "mode": "synthetic", "repository": {"id": 1, "name": "synthetic/kineti"},
              "workflow": workflow, "controller_sha256": h, "tests_sha256": h,
              "toolchain_sha256": h, "settings_sha256": h, "jobs": declarations,
              "reviewers": ["independent-fixture"], "coding_principal": "coding-fixture",
              "destinations": {stage: "synthetic:" + stage for stage in ("cargo", "npm", "github")},
              "runners": runners}
    jobs = [dict(declarations["native"], id=10, name="native", conclusion="success",
                 commit="c" * 40, run_id=20, attempt=1)]
    artifacts = []
    for stage, name in (("cargo", "kineti-cli-1.2.3-test.crate"), ("npm", "kineti-1.2.3-test.tgz")):
        data = tar_bytes()
        (root / "archives" / name).write_bytes(data)
        artifacts.append({"stage": stage, "name": name, "size": len(data), "sha256": digest(data)})
    installs = [{"target": target, "route": stage, "runner": runner, "report_sha256": h,
                 "archive_sha256": artifact["sha256"], "installed_bytes": 1234,
                 **{key: True for key in ("outside_checkout", "node_bun_absent", "mcp", "self_test",
                                          "scripts_disabled_error", "success")}}
                for target, runner in runners.items()
                for stage, artifact in (("cargo", artifacts[0]), ("npm", artifacts[1]))]
    accounting = {"snapshot_sha256": h, "decision_sha256": h, "native_exit": 0,
                  "integrity": True, "reservations": 0, "uncertain_writes": False,
                  "unknown_ids": [], "total_microcents": 55_000_000, "global_microcents": 75_000_000,
                  "stage_microcents": 55_000_000, "stage_limit_microcents": 75_000_000, "exception": None}
    m = {"schema": 1, "mode": "synthetic", "promotion_id": "b715e3dc-3ecf-4b6a-a2c4-06568850362d",
         "repository": policy["repository"], "commit": "c" * 40, "tree": "d" * 40,
         "version": "1.2.3-test", "tag": "v1.2.3-test", "workflow": workflow,
         "policy_sha256": identity(policy), "controller_sha256": h, "tests_sha256": h,
         "toolchain_sha256": h, "settings_sha256": h, "run_id": 20, "attempt": 1,
         "created_at": NOW - 60, "evidence_at": NOW - 120, "evidence_expires_at": NOW + 120,
         "jobs": jobs, "artifacts": artifacts, "installs": installs, "accounting": accounting,
         "destinations": policy["destinations"]}
    obs = {k: copy.deepcopy(m[k]) for k in ("repository", "commit", "tree", "workflow", "run_id", "attempt",
                                                   "settings_sha256", "accounting", "installs", "jobs")}
    obs.update(resources_zero_paid=True, isolated=True, policy_protected=True,
               available={s: True for s in ("cargo", "npm", "github")}, accounting_exception=None)
    approval = {"manifest_sha256": identity(m), "reviewer": "independent-fixture",
                "authenticated_at": NOW - 30, "expires_at": NOW + 100, "reference": "synthetic:review-1"}
    return m, SyntheticAuthority(policy, obs, approval)


def contend(root, manifest, queue):
    try:
        with Audit(root, manifest).locked():
            queue.put("acquired")
    except Denied:
        queue.put("denied")


def die_after_intent(root, manifest):
    with Audit(root, manifest).locked() as audit:
        audit.append("created")
        audit.append("publishing")
        audit.append("intent", {"stage": "cargo", "sha256": manifest["artifacts"][0]["sha256"]})
        os._exit(9)


class ControllerTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(dir=BASE)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.m, self.a = fixture(self.root)
        self.c = Controller(self.a, self.root / "audit", lambda: NOW)

    def run_manifest(self, reconcile=False):
        return self.c.promote(canonical(self.m), self.root / "archives", reconcile)

    def assert_denied(self, fresh=True):
        # Give each mutated proposal its own empty audit so a previous denial's
        # manifest binding cannot mask the validation being exercised now.
        if fresh:
            self.c.audit_root = self.root / ("audit-" + str(uuid.uuid4()))
            self.c.audit_root.mkdir()
        with self.assertRaises((Denied, OSError)):
            self.run_manifest()
        self.assertEqual(self.a.writes, [])
        self.assertEqual(self.a.credential_releases, 0)

    def reapprove(self):
        self.a.approval["manifest_sha256"] = identity(self.m)

    def test_approved_bytes_only_in_order_and_no_credentials(self):
        result = self.run_manifest()
        self.assertEqual(result["state"], "complete")
        self.assertEqual(self.a.writes, ["cargo", "npm", "github"])
        for a in self.m["artifacts"]:
            self.assertEqual(digest(self.a.sink[a["stage"]]), a["sha256"])
        self.assertEqual(self.a.credential_releases, 0)
        with self.assertRaises(Denied):
            self.run_manifest()

    def test_production_never_has_a_transport(self):
        self.m["mode"] = "production"
        self.c.authority = ProductionAuthority()
        self.assert_denied()

    def test_synthetic_adapter_cannot_be_relabelled_production(self):
        self.m["mode"] = "production"
        self.assert_denied()

    def test_json_duplicate_nested_keys_and_nonfinite(self):
        for raw in (b'{"x":1,"x":2}', b'{"x":{"a":1,"a":2}}', b'{"x":NaN}', b'{"x":Infinity}', b'\xff', b'[]bad'):
            with self.subTest(raw=raw), self.assertRaises(Denied):
                parse(raw)

    def test_strict_unknown_missing_schema_and_boolean_integer(self):
        original = copy.deepcopy(self.m)
        for mutate in (lambda m: m.update(schema=2), lambda m: m.update(schema=True),
                       lambda m: m.update(extra=0), lambda m: m.pop("tree"), lambda m: m.update(run_id=True)):
            self.m = copy.deepcopy(original)
            mutate(self.m)
            self.assert_denied()

    def test_every_non_success_conclusion_denies_even_when_observed(self):
        for conclusion in ("skipped", "neutral", "cancelled", "timed_out", "failure", "pending", "stale", "Success"):
            self.m["jobs"][0]["conclusion"] = conclusion
            self.a.observation["jobs"] = copy.deepcopy(self.m["jobs"])
            self.reapprove()
            self.assert_denied()

    def test_missing_duplicate_renamed_jobs(self):
        original = copy.deepcopy(self.m)
        for jobs in ([], [original["jobs"][0]] * 2,
                     [dict(original["jobs"][0], name="fake-success")]):
            self.m = copy.deepcopy(original)
            self.m["jobs"] = jobs
            self.assert_denied()

    def test_issuer_run_attempt_commit_and_argv_binding(self):
        original = copy.deepcopy(self.m)
        for key, value in (("issuer_id", 4), ("run_id", 21), ("attempt", 2), ("commit", "e" * 40),
                           ("argv", ["cargo", "test", "--locked"]), ("executable_sha256", "e" * 64),
                           ("assertions", 1)):
            self.m = copy.deepcopy(original)
            self.m["jobs"][0][key] = value
            self.a.observation["jobs"] = copy.deepcopy(self.m["jobs"])
            self.reapprove()
            self.assert_denied()

    def test_no_assertions_or_printed_success(self):
        self.m["jobs"][0].update(assertions=0, argv=["printf", "52 tests passed"])
        self.assert_denied()

    def test_expired_future_and_excess_evidence_age(self):
        original = copy.deepcopy(self.m)
        for key, value in (("evidence_at", NOW + 1), ("created_at", NOW + 1),
                           ("evidence_expires_at", NOW), ("evidence_expires_at", NOW + 14401)):
            self.m = copy.deepcopy(original)
            self.m[key] = value
            self.reapprove()
            self.assert_denied()

    def test_forged_stale_wrong_reviewer_approval(self):
        original = copy.deepcopy(self.a.approval)
        for key, value in (("manifest_sha256", "f" * 64), ("expires_at", NOW),
                           ("reviewer", "coding-fixture"), ("reviewer", "unlisted"),
                           ("authenticated_at", NOW + 1), ("expires_at", NOW + 86401)):
            self.a.approval = dict(original, **{key: value})
            self.assert_denied()

    def test_policy_and_live_settings_changed(self):
        self.a.observation["settings_sha256"] = "e" * 64
        self.assert_denied()

    def test_revoked_reviewer(self):
        self.a.policy["reviewers"] = []
        self.assert_denied()

    def test_resource_isolation_api_and_policy_fail_closed(self):
        original = copy.deepcopy(self.a.observation)
        for key in ("resources_zero_paid", "isolated", "policy_protected"):
            self.a.observation = dict(original, **{key: False})
            self.assert_denied()
        self.a.outage = True
        self.assert_denied()

    def test_occupied_version_or_tag(self):
        for stage in ("cargo", "npm", "github"):
            self.a.observation["available"][stage] = False
            self.assert_denied()

    def test_report_inventory_footprint_and_runtime_failures(self):
        original = copy.deepcopy(self.m)
        for key, value in (("installed_bytes", 10_000_000), ("node_bun_absent", False),
                           ("mcp", False), ("success", False), ("target", "windows-x64"),
                           ("archive_sha256", "f" * 64), ("runner", "emulated")):
            self.m = copy.deepcopy(original)
            self.m["installs"][0][key] = value
            self.reapprove()
            self.assert_denied()

    def test_accounting_absolute_stops(self):
        original = copy.deepcopy(self.m)
        for key, value in (("integrity", False), ("reservations", 1), ("uncertain_writes", True),
                           ("snapshot_sha256", "f" * 64), ("total_microcents", 71_250_000),
                           ("stage_microcents", 71_250_000), ("global_microcents", 0)):
            self.m = copy.deepcopy(original)
            self.m["accounting"][key] = value
            self.reapprove()
            self.assert_denied()

    def exception_fixture(self):
        a = self.m["accounting"]
        a.update(native_exit=3, unknown_ids=["synthetic-unresolved"])
        a["exception"] = {"promotion_id": self.m["promotion_id"], "snapshot_sha256": a["snapshot_sha256"],
                          "artifacts_sha256": identity(self.m["artifacts"]), "unknown_ids": a["unknown_ids"],
                          "reviewer": "independent-fixture", "expires_at": NOW + 100, "reference": "synthetic:exception"}
        self.a.observation["accounting"] = copy.deepcopy(a)
        self.a.observation["accounting_exception"] = copy.deepcopy(a["exception"])
        self.reapprove()

    def test_exact_synthetic_exception_retains_native_failure(self):
        self.exception_fixture()
        self.assertEqual(self.run_manifest()["state"], "complete")
        self.assertEqual(self.m["accounting"]["native_exit"], 3)

    def test_unknowns_without_exception_deny(self):
        self.m["accounting"].update(native_exit=3, unknown_ids=["synthetic-unknown"])
        self.a.observation["accounting"] = copy.deepcopy(self.m["accounting"])
        self.reapprove()
        self.assert_denied()

    def test_wrong_exception_scope_and_reservation_never_exempt(self):
        self.exception_fixture()
        original = copy.deepcopy(self.m)
        for key, value in (("promotion_id", "other"), ("snapshot_sha256", "f" * 64),
                           ("artifacts_sha256", "f" * 64), ("expires_at", NOW),
                           ("unknown_ids", []), ("reviewer", "coding-fixture")):
            self.m = copy.deepcopy(original)
            self.m["accounting"]["exception"][key] = value
            self.reapprove()
            self.assert_denied()
        self.m = original
        self.m["accounting"]["reservations"] = 1
        self.assert_denied()

    def test_archive_swap_and_symlink_deny(self):
        entry = self.m["artifacts"][0]
        path = self.root / "archives" / entry["name"]
        path.write_bytes(b"swapped")
        self.assert_denied()
        path.unlink()
        path.symlink_to(self.root / "missing")
        self.assert_denied()

    def test_archive_traversal_links_devices_and_duplicate_paths(self):
        for name, kind in (("../outside", None), ("/absolute", None), ("x\\y", None),
                           ("package/link", tarfile.SYMTYPE), ("package/hard", tarfile.LNKTYPE),
                           ("package/device", tarfile.CHRTYPE)):
            data = tar_bytes(name, kind)
            path = self.root / "archives" / "unsafe.tgz"
            path.write_bytes(data)
            with self.subTest(name=name), self.assertRaises(Denied):
                archive_bytes(path.parent, {"name": path.name, "size": len(data), "sha256": digest(data)})

    def test_archive_name_traversal_unexpected_fields(self):
        self.m["artifacts"][0]["name"] = "../unsafe"
        self.assert_denied()

    def test_unexpected_staged_artifact_denies(self):
        (self.root / "archives" / "unapproved.crate").write_bytes(b"unapproved")
        self.assert_denied()

    def test_duplicate_normalized_archive_paths_deny(self):
        for name in ("package/./file", "package//file", "./package/file"):
            data = tar_bytes(name)
            path = self.root / "unsafe.tgz"
            path.write_bytes(data)
            with self.assertRaises(Denied):
                archive_bytes(path.parent, {"name": path.name, "size": len(data), "sha256": digest(data)})

    def test_job_killed_after_intent_requires_inspection(self):
        # Simulate process death without executing exception handling. Durable
        # intent is sufficient to block a restarted controller's default path.
        audit = Audit(self.root / "audit", self.m)
        with audit.locked():
            audit.append("created")
            audit.append("validated")
            audit.append("approved")
            audit.append("publishing")
            audit.append("intent", {"stage": "cargo", "sha256": self.m["artifacts"][0]["sha256"]})
        with self.assertRaises(Denied):
            self.run_manifest()
        self.assertEqual(self.a.writes, [])

    def test_publisher_revocation_blocks_remaining_stages(self):
        publish = self.a.publish
        def revoke(stage, data):
            publish(stage, data)
            self.a.outage = True
        self.a.publish = revoke
        with self.assertRaises(Denied):
            self.run_manifest()
        self.assertEqual(self.a.writes, ["cargo"])

    def test_after_each_upload_loss_reconciles_without_duplicate(self):
        for stage in ("cargo", "npm", "github"):
            with self.subTest(stage=stage), tempfile.TemporaryDirectory(dir=BASE) as root:
                m, a = fixture(root)
                c = Controller(a, Path(root) / "audit", lambda: NOW)
                a.fail_stage, a.after_write = stage, True
                with self.assertRaises(OSError):
                    c.promote(canonical(m), Path(root) / "archives")
                before = list(a.writes)
                with self.assertRaises(Denied):
                    c.promote(canonical(m), Path(root) / "archives")
                self.assertEqual(a.writes, before)
                a.fail_stage = None
                c.promote(canonical(m), Path(root) / "archives", reconcile=True)
                self.assertEqual(a.writes, ["cargo", "npm", "github"])

    def test_network_failure_before_upload_requires_reconciliation(self):
        self.a.fail_stage = "cargo"
        with self.assertRaises(OSError):
            self.run_manifest()
        self.assertEqual(self.a.writes, [])
        with self.assertRaises(Denied):
            self.run_manifest()
        self.a.fail_stage = None
        self.run_manifest(reconcile=True)
        self.assertEqual(self.a.writes, ["cargo", "npm", "github"])

    def test_uncertain_registry_digest_mismatch_never_resumes(self):
        self.a.fail_stage, self.a.after_write = "cargo", True
        with self.assertRaises(OSError):
            self.run_manifest()
        self.a.sink["cargo"] = b"different registry bytes"
        with self.assertRaises(Denied):
            self.run_manifest(reconcile=True)
        self.assertEqual(self.a.writes, ["cargo"])

    def test_expired_approval_blocks_partial_remainder(self):
        self.a.fail_stage, self.a.after_write = "cargo", True
        with self.assertRaises(OSError):
            self.run_manifest()
        self.c.clock = lambda: NOW + 200
        with self.assertRaises(Denied):
            self.run_manifest(reconcile=True)
        self.assertEqual(self.a.writes, ["cargo"])

    def test_stage_revalidation_stops_settings_change(self):
        publish = self.a.publish
        def mutate(stage, data):
            publish(stage, data)
            self.a.observation["settings_sha256"] = "e" * 64
        self.a.publish = mutate
        with self.assertRaises(Denied):
            self.run_manifest()
        self.assertEqual(self.a.writes, ["cargo"])

    def test_audit_missing_and_corrupt_checkpoint_denies(self):
        audit = Audit(self.root / "audit", self.m)
        with audit.locked():
            audit.append("created")
        (audit.folder / "head").write_bytes(b"tampered")
        self.assert_denied(fresh=False)
        (audit.folder / "head").unlink()
        self.assert_denied(fresh=False)

    def test_partial_audit_line_denies(self):
        audit = Audit(self.root / "audit", self.m)
        with audit.locked():
            audit.append("created")
        with (audit.folder / "events").open("ab") as stream:
            stream.write(b'{"unfinished":')
        self.assert_denied(fresh=False)

    def test_audit_persistence_failure_prevents_next_write(self):
        original = Audit.append
        def fail(audit, event, payload=None):
            if event == "verified":
                raise OSError("disk failure")
            return original(audit, event, payload)
        with patch.object(Audit, "append", fail), self.assertRaises(OSError):
            self.run_manifest()
        self.assertEqual(self.a.writes, ["cargo"])
        with self.assertRaises(Denied):
            self.run_manifest()

    def test_intent_persistence_failure_grants_no_write(self):
        original = Audit.append
        def fail(audit, event, payload=None):
            if event == "intent":
                raise OSError("disk failure")
            return original(audit, event, payload)
        with patch.object(Audit, "append", fail), self.assertRaises(OSError):
            self.run_manifest()
        self.assertEqual(self.a.writes, [])

    def test_cross_process_package_version_lock(self):
        ctx = multiprocessing.get_context("spawn")
        queue = ctx.Queue()
        with Audit(self.root / "audit", self.m).locked():
            child = ctx.Process(target=contend, args=(self.root / "audit", self.m, queue))
            child.start()
            self.assertEqual(queue.get(timeout=10), "denied")
            child.join(timeout=10)
            self.assertEqual(child.exitcode, 0)
        queue.close()

    def test_changed_promotion_for_same_version_never_rebinds_audit(self):
        self.run_manifest()
        self.m["promotion_id"] = "b715e3dc-3ecf-4b6a-a2c4-06568850362e"
        self.reapprove()
        with self.assertRaises(Denied):
            self.run_manifest()
        self.assertEqual(self.a.writes, ["cargo", "npm", "github"])

    def test_actual_process_death_leaves_durable_intent(self):
        ctx = multiprocessing.get_context("spawn")
        child = ctx.Process(target=die_after_intent, args=(self.root / "audit", self.m))
        child.start()
        child.join(timeout=10)
        self.assertEqual(child.exitcode, 9)
        self.assert_denied(fresh=False)

    def test_fsync_append_before_checkpoint_interruption_denies(self):
        audit = Audit(self.root / "audit", self.m)
        with audit.locked():
            audit.append("created")
            with patch("controller.os.replace", side_effect=OSError("interrupted checkpoint")):
                with self.assertRaises(OSError):
                    audit.append("validated")
        self.assert_denied(fresh=False)

    def test_checked_in_schema_matches_manifest_and_approval_keys(self):
        folder = Path(__file__).parent / "schemas"
        manifest = json.loads((folder / "manifest.schema.json").read_text())
        approval = json.loads((folder / "approval.schema.json").read_text())
        self.assertEqual(set(manifest["properties"]), set(self.m))
        self.assertEqual(set(approval["properties"]), set(self.a.approval))
        self.assertFalse(manifest["additionalProperties"])

    def test_changed_source_repository_workflow_and_tree_deny(self):
        original = copy.deepcopy(self.m)
        for key, value in (("repository", {"id": 999, "name": "fork/kineti"}),
                           ("commit", "e" * 40), ("tree", "e" * 40),
                           ("workflow", dict(self.m["workflow"], revision="e" * 40)),
                           ("policy_sha256", "e" * 64), ("tests_sha256", "e" * 64),
                           ("toolchain_sha256", "e" * 64)):
            self.m = copy.deepcopy(original)
            self.m[key] = value
            self.reapprove()
            self.assert_denied()

    def test_missing_entire_initialized_audit_directory_never_resets(self):
        audit = Audit(self.root / "audit", self.m)
        with audit.locked():
            audit.append("created")
            audit.append("intent", {"stage": "cargo"})
        shutil.rmtree(audit.folder)
        self.assert_denied(fresh=False)

    def test_same_registry_version_locks_across_repository_identity(self):
        changed = copy.deepcopy(self.m)
        changed["repository"] = {"id": 999, "name": "other/repository"}
        self.assertEqual(Audit(self.root / "audit", changed).key,
                         Audit(self.root / "audit", self.m).key)


if __name__ == "__main__":
    unittest.main(verbosity=2)

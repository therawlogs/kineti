"""Internal, offline release-control prototype. No production credential adapter.

Run only under independently controlled policy before granting real authority.
The synthetic adapter proves local decisions, never provider enforcement.
"""
import copy
import fcntl
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import tarfile
import time
import uuid
from contextlib import contextmanager


class Denied(Exception):
    """No authorization; ambiguous prior writes still require reconciliation."""


def require(condition, message):
    if not condition:
        raise Denied(message)


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=True, allow_nan=False).encode("ascii")


def digest(value):
    return hashlib.sha256(value).hexdigest()


def identity(value):
    return digest(canonical(value))


def parse(raw):
    """Reject duplicate keys and nonfinite numbers before any side effect."""
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, "duplicate JSON key")
            result[key] = value
        return result

    def constant(_):
        raise Denied("nonfinite JSON number")
    require(isinstance(raw, bytes) and len(raw) <= 2_000_000, "JSON size/type")
    try:
        return json.loads(raw, object_pairs_hook=pairs, parse_constant=constant)
    except (ValueError, UnicodeError, RecursionError) as error:
        raise Denied("malformed JSON") from error


def fields(value, names):
    require(type(value) is dict and set(value) == set(names.split()),
            "missing or unknown fields: " + names)


def integer(value, minimum=0):
    require(type(value) is int and minimum <= value <= 9_007_199_254_740_991,
            "invalid integer")


def token(value):
    require(type(value) is str and bool(re.fullmatch(r"[A-Za-z0-9_.:/@+-]{1,240}", value)),
            "invalid identity string")


def hash256(value):
    require(type(value) is str and bool(re.fullmatch(r"[a-f0-9]{64}", value)), "SHA-256 required")


def git_hash(value):
    require(type(value) is str and bool(re.fullmatch(r"[a-f0-9]{40}", value)), "Git identity required")


TARGETS = {"linux-x64", "linux-arm64", "darwin-x64", "darwin-arm64"}
STAGES = ("cargo", "npm", "github")


def schema(m):
    fields(m, "schema mode promotion_id repository commit tree version tag workflow policy_sha256 "
           "controller_sha256 tests_sha256 toolchain_sha256 settings_sha256 run_id attempt "
           "created_at evidence_at evidence_expires_at jobs artifacts installs accounting destinations")
    require(m["schema"] == 1 and type(m["schema"]) is int, "unsupported manifest schema")
    require(m["mode"] in ("synthetic", "production"), "unsupported mode")
    try:
        require(str(uuid.UUID(m["promotion_id"])) == m["promotion_id"], "promotion UUID")
    except (ValueError, AttributeError, TypeError) as error:
        raise Denied("promotion UUID") from error
    fields(m["repository"], "id name")
    integer(m["repository"]["id"], 1)
    token(m["repository"]["name"])
    for name in ("commit", "tree"):
        git_hash(m[name])
    require(type(m["version"]) is str and bool(re.fullmatch(
        r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-[a-z0-9.-]+)?", m["version"])), "exact version")
    require(m["tag"] == "v" + m["version"], "tag/version mismatch")
    fields(m["workflow"], "repository_id path revision issuer_id environment")
    for name in ("repository_id", "issuer_id"):
        integer(m["workflow"][name], 1)
    git_hash(m["workflow"]["revision"])
    token(m["workflow"]["path"])
    token(m["workflow"]["environment"])
    for name in ("policy_sha256", "controller_sha256", "tests_sha256", "toolchain_sha256", "settings_sha256"):
        hash256(m[name])
    for name in ("run_id", "attempt"):
        integer(m[name], 1)
    for name in ("created_at", "evidence_at", "evidence_expires_at"):
        integer(m[name], 1)
    require(type(m["jobs"]) is list and 0 < len(m["jobs"]) <= 100, "job inventory")
    seen = set()
    for job in m["jobs"]:
        fields(job, "id name issuer_id conclusion commit run_id attempt runner assertions argv executable_sha256")
        for name in ("id", "issuer_id", "run_id", "attempt", "assertions"):
            integer(job[name], 1)
        for name in ("name", "runner", "conclusion"):
            token(job[name])
        git_hash(job["commit"])
        hash256(job["executable_sha256"])
        require(type(job["argv"]) is list and len(job["argv"]) > 0 and
                all(type(x) is str and "\0" not in x for x in job["argv"]), "argv array required")
        require(job["id"] not in seen, "duplicate job")
        seen.add(job["id"])
    require(type(m["artifacts"]) is list and len(m["artifacts"]) == 2, "exact archive inventory")
    seen = set()
    for archive in m["artifacts"]:
        fields(archive, "stage name size sha256")
        require(archive["stage"] in ("cargo", "npm") and archive["stage"] not in seen, "archive stage")
        seen.add(archive["stage"])
        require(type(archive["name"]) is str and bool(re.fullmatch(r"[a-zA-Z0-9_.-]+", archive["name"])), "unsafe archive name")
        integer(archive["size"], 1)
        require(archive["size"] <= 64_000_000, "archive size limit")
        hash256(archive["sha256"])
    require(type(m["installs"]) is list and len(m["installs"]) == 8, "eight installs required")
    seen = set()
    for report in m["installs"]:
        fields(report, "target route runner report_sha256 archive_sha256 installed_bytes outside_checkout "
               "node_bun_absent mcp self_test scripts_disabled_error success")
        key = (report["target"], report["route"])
        require(key[0] in TARGETS and key[1] in ("cargo", "npm") and key not in seen, "install inventory")
        seen.add(key)
        token(report["runner"])
        hash256(report["report_sha256"])
        hash256(report["archive_sha256"])
        integer(report["installed_bytes"], 1)
        require(report["installed_bytes"] < 10_000_000, "footprint limit")
        for name in ("outside_checkout", "node_bun_absent", "mcp", "self_test", "scripts_disabled_error", "success"):
            require(report[name] is True, "failed installation: " + name)
    fields(m["accounting"], "snapshot_sha256 decision_sha256 native_exit integrity reservations uncertain_writes "
           "unknown_ids total_microcents global_microcents stage_microcents stage_limit_microcents exception")
    a = m["accounting"]
    for name in ("snapshot_sha256", "decision_sha256"):
        hash256(a[name])
    for name in ("native_exit", "reservations", "total_microcents", "global_microcents", "stage_microcents", "stage_limit_microcents"):
        integer(a[name])
    require(a["integrity"] is True and a["uncertain_writes"] is False, "accounting integrity/uncertainty")
    require(a["reservations"] == 0, "unresolved monetary reservations")
    require(type(a["unknown_ids"]) is list and all(type(x) is str for x in a["unknown_ids"])
            and len(set(a["unknown_ids"])) == len(a["unknown_ids"]), "unknown inventory")
    require(a["total_microcents"] * 100 < a["global_microcents"] * 95 and
            a["stage_microcents"] * 100 < a["stage_limit_microcents"] * 95, "95 percent spending stop")
    require(type(m["destinations"]) is dict and set(m["destinations"]) == set(STAGES), "destination inventory")
    for value in m["destinations"].values():
        token(value)


def validate(m, policy, observation, approval, now):
    """Policy, observation and approval must come from an independent adapter.

    The current implementation has ONLY a synthetic adapter. Caller-supplied JSON
    claiming authentication can never unlock a production transport.
    """
    schema(m)
    integer(now, 1)
    fields(policy, "schema mode repository workflow controller_sha256 tests_sha256 toolchain_sha256 "
           "settings_sha256 jobs reviewers coding_principal destinations runners")
    require(type(policy["schema"]) is int and policy["schema"] == 1, "unsupported policy schema")
    require(type(policy["reviewers"]) is list and policy["reviewers"] and
            len(set(policy["reviewers"])) == len(policy["reviewers"]) and
            policy["coding_principal"] not in policy["reviewers"], "reviewer separation")
    fields(observation, "repository commit tree workflow run_id attempt settings_sha256 accounting installs "
           "jobs resources_zero_paid isolated policy_protected available accounting_exception")
    require(m["policy_sha256"] == identity(policy), "policy digest differs")
    for name in ("repository", "workflow", "controller_sha256", "tests_sha256", "toolchain_sha256", "destinations"):
        require(m[name] == policy[name], "policy identity differs: " + name)
    require(m["mode"] == policy["mode"], "mode differs")
    require(m["evidence_at"] <= m["created_at"] <= now and
            m["evidence_at"] <= now < m["evidence_expires_at"] <= m["evidence_at"] + 240 * 60,
            "future or expired evidence")
    require(m["jobs"] == observation["jobs"], "job observations differ")
    expected = policy["jobs"]
    require(len(m["jobs"]) == len(expected) and len({j["name"] for j in m["jobs"]}) == len(expected), "required jobs missing/duplicate")
    for job in m["jobs"]:
        require(job["name"] in expected, "unexpected job")
        declaration = expected[job["name"]]
        require(job["conclusion"] == "success", "job did not succeed")
        for name in ("commit", "run_id", "attempt"):
            require(job[name] == m[name], "job origin differs")
        for name in ("issuer_id", "runner", "argv", "executable_sha256", "assertions"):
            require(job[name] == declaration[name], "job declaration differs: " + name)
    for name in ("repository", "commit", "tree", "workflow", "run_id", "attempt", "settings_sha256", "accounting", "installs"):
        require(m[name] == observation[name], "live observation differs: " + name)
    require(observation["resources_zero_paid"] is True, "resource/billing uncertainty")
    require(observation["isolated"] is True and observation["policy_protected"] is True, "trust boundary unavailable")
    require(observation["settings_sha256"] == policy["settings_sha256"], "effective settings changed")
    require(observation["available"] == {stage: True for stage in STAGES}, "version or tag occupied")
    require(m["installs"] == observation["installs"], "installation evidence differs")
    archives = {a["stage"]: a for a in m["artifacts"]}
    for report in m["installs"]:
        require(report["archive_sha256"] == archives[report["route"]]["sha256"], "report/archive mismatch")
        require(report["runner"] == policy["runners"][report["target"]], "native runner differs")
    require(archives["cargo"]["name"] == "kineti-cli-" + m["version"] + ".crate" and
            archives["npm"]["name"] == "kineti-" + m["version"] + ".tgz", "package/version differs")
    fields(approval, "manifest_sha256 reviewer authenticated_at expires_at reference")
    require(approval["manifest_sha256"] == identity(m), "approval is not bound to manifest")
    require(approval["reviewer"] in policy["reviewers"] and approval["reviewer"] != policy["coding_principal"], "independent reviewer required")
    for name in ("authenticated_at", "expires_at"):
        integer(approval[name], 1)
    token(approval["reference"])
    require(m["created_at"] <= approval["authenticated_at"] <= now < approval["expires_at"] <=
            min(m["evidence_expires_at"], approval["authenticated_at"] + 86400), "approval expired/future")
    a = m["accounting"]
    if a["unknown_ids"]:
        e = a["exception"]
        fields(e, "promotion_id snapshot_sha256 artifacts_sha256 unknown_ids reviewer expires_at reference")
        require(a["native_exit"] == 3 and e["promotion_id"] == m["promotion_id"] and
                e["snapshot_sha256"] == a["snapshot_sha256"] and
                e["artifacts_sha256"] == identity(m["artifacts"]) and
                e["unknown_ids"] == a["unknown_ids"] and e["reviewer"] in policy["reviewers"] and
                e["reviewer"] != policy["coding_principal"] and type(e["expires_at"]) is int and
                now < e["expires_at"] <= approval["expires_at"], "accounting exception scope differs")
        require(e == observation["accounting_exception"], "exception is not independently authenticated")
    else:
        require(a["native_exit"] == 0 and a["exception"] is None, "accounting decision invalid")


def archive_bytes(root, entry):
    """Read with no symlink following; inspect without extracting or running hooks."""
    root = Path(root)
    require(not root.is_symlink(), "staging root symlink")
    fd = os.open(str(root / entry["name"]), os.O_RDONLY | os.O_NOFOLLOW)
    try:
        info = os.fstat(fd)
        require(stat.S_ISREG(info.st_mode) and info.st_size == entry["size"], "archive size/type differs")
        with os.fdopen(fd, "rb", closefd=False) as stream:
            data = stream.read(entry["size"] + 1)
    finally:
        os.close(fd)
    require(len(data) == entry["size"] and digest(data) == entry["sha256"], "archive hash differs")
    names = set()
    total = 0
    try:
        with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
            for index, member in enumerate(archive):
                path = PurePosixPath(member.name)
                normalized = path.as_posix()
                require(index < 10000 and normalized not in names and
                        member.name.rstrip("/") == normalized and not path.is_absolute() and
                        ".." not in path.parts and "\\" not in member.name and
                        all(ord(c) >= 32 for c in member.name) and bool(path.parts), "unsafe archive path")
                require(member.isfile() or member.isdir(), "archive links/devices forbidden")
                require(not member.mode & 0o7000, "unsafe archive permissions")
                names.add(normalized)
                total += member.size
                require(total <= 128_000_000, "archive expansion limit")
    except (tarfile.TarError, EOFError, OSError) as error:
        raise Denied("malformed archive") from error
    require(names, "empty archive")
    return data


def sync_directory(path):
    fd = os.open(str(path), os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def exclusive_write(path, data):
    fd = os.open(str(path), os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    try:
        with os.fdopen(fd, "wb", closefd=False) as stream:
            stream.write(data)
            stream.flush()
            os.fsync(fd)
    finally:
        os.close(fd)
    sync_directory(path.parent)


class Audit:
    """Durable audit with torn-write detection; host isolation remains required."""
    def __init__(self, root, manifest, clock=None):
        self.root = Path(root)
        require(self.root.is_dir() and not self.root.is_symlink(), "audit root must be provisioned")
        # Lock registry coordinates, not a caller-chosen promotion/repository ID.
        self.key = identity({"version": manifest["version"],
                             "cargo": manifest["destinations"]["cargo"],
                             "npm": manifest["destinations"]["npm"]})
        self.manifest_hash = identity(manifest)
        self.clock = clock or (lambda: int(time.time()))
        self.folder = self.root / self.key

    @contextmanager
    def locked(self):
        fd = os.open(str(self.root / (self.key + ".lock")), os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
        try:
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise Denied("promotion already locked") from error
            marker = self.root / (self.key + ".initialized")
            if not marker.exists():
                require(not self.folder.exists(), "orphan audit directory; inspect")
                exclusive_write(marker, self.manifest_hash.encode())
                self.folder.mkdir(mode=0o700)
                sync_directory(self.root)
                exclusive_write(self.folder / "manifest", self.manifest_hash.encode())
                exclusive_write(self.folder / "events", b"")
                exclusive_write(self.folder / "head", identity([]).encode())
            require(self.folder.is_dir(), "initialized audit directory missing; inspect before replay")
            try:
                marker_fd = os.open(str(marker), os.O_RDONLY | os.O_NOFOLLOW)
                with os.fdopen(marker_fd, "rb") as stream:
                    require(stream.read() == self.manifest_hash.encode(), "initialized manifest differs")
            except OSError as error:
                raise Denied("audit initialization marker damaged") from error
            require(not self.folder.is_symlink(), "audit symlink")
            require(self.read("manifest") == self.manifest_hash.encode(), "package/version already bound to another manifest")
            self.events = self.load()
            yield self
        finally:
            os.close(fd)

    def read(self, name):
        try:
            fd = os.open(str(self.folder / name), os.O_RDONLY | os.O_NOFOLLOW)
            with os.fdopen(fd, "rb") as stream:
                require(stat.S_ISREG(os.fstat(stream.fileno()).st_mode), "audit file type")
                return stream.read()
        except OSError as error:
            raise Denied("audit persistence missing; inspect exact state") from error

    def load(self):
        raw = self.read("events")
        require(not raw or raw.endswith(b"\n"), "interrupted audit append")
        events = [parse(line) for line in raw.splitlines()]
        previous = "GENESIS"
        for index, row in enumerate(events):
            fields(row, "sequence previous manifest_sha256 at actor event payload sha256")
            integer(row["at"], 1)
            require(row["actor"] == "synthetic-controller", "audit actor differs")
            hashed = {k: v for k, v in row.items() if k != "sha256"}
            require(row["sequence"] == index and row["previous"] == previous and
                    row["manifest_sha256"] == self.manifest_hash and row["sha256"] == identity(hashed), "audit chain differs")
            previous = row["sha256"]
        require(self.read("head") == identity(events).encode(), "audit checkpoint differs; inspection required")
        return events

    def append(self, event, payload=None):
        row = {"sequence": len(self.events), "previous": self.events[-1]["sha256"] if self.events else "GENESIS",
               "manifest_sha256": self.manifest_hash, "at": self.clock(),
               "actor": "synthetic-controller", "event": event, "payload": payload}
        row["sha256"] = identity(row)
        fd = os.open(str(self.folder / "events"), os.O_WRONLY | os.O_APPEND | os.O_NOFOLLOW)
        try:
            data = canonical(row) + b"\n"
            require(os.write(fd, data) == len(data), "short audit append")
            os.fsync(fd)
        finally:
            os.close(fd)
        self.events.append(row)
        temp = self.folder / ("head-" + str(uuid.uuid4()))
        exclusive_write(temp, identity(self.events).encode())
        os.replace(str(temp), str(self.folder / "head"))
        sync_directory(self.folder)


class ProductionAuthority:
    """Unavailable until independently reviewed identity and provider adapters exist."""
    def snapshot(self, manifest):
        raise Denied("production disabled: independent authority, provider claims and exact-byte upload unverified")


class SyntheticAuthority:
    """Offline test fixture only. Never grant OIDC, secrets, or network access."""
    def __init__(self, policy, observation, approval):
        require(policy["mode"] == "synthetic", "synthetic policy required")
        self.policy = copy.deepcopy(policy)
        self.observation = copy.deepcopy(observation)
        self.approval = copy.deepcopy(approval)
        self.sink = {}
        self.writes = []
        self.credential_releases = 0
        self.fail_stage = None
        self.after_write = False
        self.outage = False

    def snapshot(self, manifest):
        require(not self.outage, "authority unavailable")
        require(manifest["mode"] == "synthetic" and all(
            x.startswith("synthetic:") for x in manifest["destinations"].values()), "synthetic destinations required")
        return copy.deepcopy(self.policy), copy.deepcopy(self.observation), copy.deepcopy(self.approval)

    def publish(self, stage, data):
        require(stage not in self.sink, "duplicate synthetic write")
        if stage == self.fail_stage and not self.after_write:
            raise OSError("synthetic network uncertainty before write")
        self.sink[stage] = bytes(data)
        self.writes.append(stage)
        if stage == self.fail_stage:
            raise OSError("synthetic lost response after write")

    def downloaded(self, stage):
        require(not self.outage, "registry unavailable")
        return self.sink.get(stage)


class Controller:
    """No command-line entry point, production transport or credential discovery."""
    def __init__(self, authority, audit_root, clock):
        self.authority = authority
        self.audit_root = Path(audit_root)
        self.clock = clock

    def _validate(self, m, archives):
        policy, observation, approval = self.authority.snapshot(m)
        validate(m, policy, observation, approval, self.clock())
        require(type(self.authority) is SyntheticAuthority and m["mode"] == "synthetic",
                "production transport disabled")
        require(set(os.listdir(archives)) == {a["name"] for a in m["artifacts"]}, "unexpected staged files")
        payloads = {entry["stage"]: archive_bytes(archives, entry) for entry in m["artifacts"]}
        payloads["github"] = canonical({"tag": m["tag"], "commit": m["commit"], "artifacts": m["artifacts"]})
        return payloads, approval

    def promote(self, raw, archives, reconcile=False):
        m = parse(raw)
        schema(m)
        audit = Audit(self.audit_root, m, self.clock)
        with audit.locked():
            events = audit.events
            require(not events or events[-1]["event"] != "complete", "promotion already complete")
            has_intent = any(e["event"] == "intent" for e in events)
            require(not has_intent or reconcile, "prior publication intent requires explicit reconciliation")
            if not events:
                audit.append("created")
            try:
                payloads, approval = self._validate(m, archives)
            except Exception as error:
                audit.append("partial" if has_intent else "denied", {"reason": type(error).__name__})
                raise
            audit.append("validated", {"policy_sha256": m["policy_sha256"],
                                       "settings_sha256": m["settings_sha256"],
                                       "accounting": m["accounting"]})
            audit.append("approved", approval)
            completed = set()
            if has_intent:
                # Reconcile every service before any new write, including operations
                # whose response was lost. None here means definite absence only in
                # this offline sink; a production adapter must prove that distinction.
                for stage in STAGES:
                    got = self.authority.downloaded(stage)
                    if got is not None:
                        require(got == payloads[stage], "registry bytes differ; incident inspection required")
                        completed.add(stage)
                    elif any(e["event"] == "verified" and e["payload"]["stage"] == stage for e in events):
                        raise Denied("previously verified bytes now absent")
                audit.append("reconciled", {"stages": sorted(completed)})
            audit.append("publishing")
            for stage in STAGES:
                if stage in completed:
                    continue
                try:
                    # Fresh policy/approval/accounting and archive rehash at each
                    # boundary; pass immutable bytes from this read to the sink.
                    fresh, current_approval = self._validate(m, archives)
                    require(fresh == payloads, "staged bytes changed")
                    require(current_approval == approval, "approval changed at publication boundary")
                    audit.append("intent", {"stage": stage, "sha256": digest(fresh[stage])})
                    self.authority.publish(stage, fresh[stage])
                    require(self.authority.downloaded(stage) == fresh[stage], "download differs")
                    audit.append("verified", {"stage": stage, "sha256": digest(fresh[stage])})
                    completed.add(stage)
                except BaseException:
                    # Audit failure can prevent even this append. The previous
                    # durable intent/checkpoint then blocks blind replay.
                    audit.append("uncertain", {"stage": stage, "verified": sorted(completed)})
                    raise
            audit.append("complete", {"stages": list(STAGES), "scope": "synthetic only"})
            return {"state": "complete", "mode": "synthetic", "manifest_sha256": identity(m)}

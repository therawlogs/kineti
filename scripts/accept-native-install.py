"""Exact native installation checks with disposable, explicitly selected roots."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
from pathlib import PurePosixPath
import stat
import subprocess
import tarfile
import urllib.parse
import urllib.request

VERSION = "0.6.1"


def fetch_published_archives(destination, cargo_sha256, npm_sha256):
    """Read fixed registries only and retain exact approved bytes before installs."""
    class RegistryRedirects(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, request, response, code, message, headers, location):
            parsed = urllib.parse.urlparse(location)
            if parsed.scheme != "https" or parsed.hostname not in {"crates.io", "static.crates.io", "registry.npmjs.org"}:
                raise RuntimeError("Unexpected registry download redirect destination")
            return super().redirect_request(request, response, code, message, headers, location)

    opener = urllib.request.build_opener(RegistryRedirects())

    def fetch(url):
        request = urllib.request.Request(url, headers={"User-Agent": "Kineti-published-acceptance/" + VERSION})
        with opener.open(request, timeout=60) as response:
            raw = response.read(20_000_001)
        if len(raw) > 20_000_000:
            raise RuntimeError("Published archive response exceeds the acceptance download bound")
        return raw

    cargo_metadata = json.loads(fetch("https://crates.io/api/v1/crates/kineti-cli/" + VERSION))
    npm_metadata = json.loads(fetch("https://registry.npmjs.org/kineti/" + VERSION))
    assert cargo_metadata["version"]["num"] == VERSION
    assert cargo_metadata["version"]["checksum"] == cargo_sha256
    assert npm_metadata["name"] == "kineti" and npm_metadata["version"] == VERSION
    npm_url = "https://registry.npmjs.org/kineti/-/kineti-" + VERSION + ".tgz"
    assert npm_metadata["dist"]["tarball"] == npm_url
    if destination.exists() and any(destination.iterdir()):
        raise RuntimeError("Published download directory already contains evidence; preserve before rerun")
    destination.mkdir(parents=True, exist_ok=True)
    sums = {}
    for name, url, expected in [
        ("kineti-cli-" + VERSION + ".crate", "https://crates.io/api/v1/crates/kineti-cli/" + VERSION + "/download", cargo_sha256),
        ("kineti-" + VERSION + ".tgz", npm_url, npm_sha256),
    ]:
        raw = fetch(url)
        if hashlib.sha256(raw).hexdigest() != expected:
            raise RuntimeError("Published archive differs from the exact reviewed SHA-256: " + name)
        with (destination / name).open("xb") as output:
            output.write(raw)
        sums[name] = expected
    (destination / "SHA256SUMS.json").write_text(json.dumps(sums, indent=2) + "\n")
    (destination / "registry-metadata.json").write_text(json.dumps({"version": VERSION, "cargo_checksum": cargo_sha256,
        "npm_tarball": npm_url, "npm_archive_sha256": npm_sha256, "provider_billing_verified": False}, indent=2) + "\n")

parser = argparse.ArgumentParser()
parser.add_argument("--artifacts", required=True)
parser.add_argument("--target", required=True)
parser.add_argument("--rust-target", required=True)
parser.add_argument("--work-root")
parser.add_argument("--published-registry", action="store_true")
parser.add_argument("--expected-cargo-sha256")
parser.add_argument("--expected-npm-sha256")
options = parser.parse_args()
root = Path(__file__).resolve().parents[1]
artifacts = Path(options.artifacts).resolve()
expected_hosts = {"linux-x64": ("Linux", "x86_64"), "linux-arm64": ("Linux", "aarch64"),
                  "darwin-x64": ("Darwin", "x86_64"), "darwin-arm64": ("Darwin", "arm64")}
if options.target not in expected_hosts or (os.uname().sysname, os.uname().machine) != expected_hosts[options.target]:
    parser.error("selected target does not match the actual runner OS/architecture")
expected_triples = {"linux-x64": "x86_64-unknown-linux-musl", "linux-arm64": "aarch64-unknown-linux-musl",
                    "darwin-x64": "x86_64-apple-darwin", "darwin-arm64": "aarch64-apple-darwin"}
if options.rust_target != expected_triples[options.target]:
    parser.error("Rust target must match the selected actual runner")
if options.published_registry:
    if any(not re.fullmatch(r"[a-f0-9]{64}", value or "") for value in [options.expected_cargo_sha256, options.expected_npm_sha256]):
        parser.error("published acceptance requires both exact reviewed lowercase SHA-256 archive digests")
    fetch_published_archives(artifacts, options.expected_cargo_sha256, options.expected_npm_sha256)
elif options.expected_cargo_sha256 or options.expected_npm_sha256:
    parser.error("reviewed registry digests require --published-registry")
declared_archives = json.loads((artifacts / "SHA256SUMS.json").read_text())
if set(declared_archives) != {"kineti-cli-" + VERSION + ".crate", "kineti-" + VERSION + ".tgz"}:
    raise RuntimeError("Acceptance requires exactly the two fixed-version archives")
for name, expected in declared_archives.items():
    if hashlib.sha256((artifacts / name).read_bytes()).hexdigest() != expected:
        raise RuntimeError("Candidate artifact digest mismatch: " + name)
work = (Path(options.work_root).resolve() if options.work_root else root / ".kineti/install-acceptance") / options.target
if options.work_root and (work == root or root in work.parents):
    raise RuntimeError("Native runner acceptance must use a directory outside the source checkout")
if work.exists():
    raise RuntimeError("Acceptance directory already exists; preserve and inspect before rerun")
work.mkdir(parents=True)
source = work / "source"
source.mkdir()
crate = artifacts / "kineti-cli-0.6.1.crate"
with tarfile.open(crate) as archive:
    for member in archive.getmembers():
        relative = PurePosixPath(member.name)
        if relative.is_absolute() or ".." in relative.parts or member.issym() or member.islnk():
            raise RuntimeError("Unsafe crate archive path")
        destination = source.joinpath(*relative.parts)
        if member.isdir():
            destination.mkdir(parents=True, exist_ok=True)
        elif member.isfile():
            destination.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as data, destination.open("wb") as output:
                shutil.copyfileobj(data, output)
        else:
            raise RuntimeError("Unsupported crate archive entry")
crate_source = source / "kineti-cli-0.6.1"
env = dict(os.environ, CARGO_TARGET_DIR=str(work / "cargo-target"), npm_config_cache=str(work / "npm-cache"))
archive = artifacts / "kineti-0.6.1.tgz"
with tarfile.open(archive) as packed:
    for member in packed.getmembers():
        relative = PurePosixPath(member.name)
        if relative.is_absolute() or ".." in relative.parts or not (member.isfile() or member.isdir()):
            raise RuntimeError("Unsafe npm archive entry")
cargo_root = work / "cargo-install"
cargo_command = (["cargo", "install", "kineti-cli", "--version", "=" + VERSION, "--registry", "crates-io"]
                 if options.published_registry else ["cargo", "install", "--path", str(crate_source)])
cargo_command += ["--locked", "--target", options.rust_target, "--root", str(cargo_root)]
subprocess.run(cargo_command, env=env, check=True)
npm_root = work / "npm-install"
npm_spec = ["--registry", "https://registry.npmjs.org", "kineti@" + VERSION] if options.published_registry else [str(archive)]
npm_command = ["npm", "install", "--global", "--prefix", str(npm_root), "--no-audit", "--no-fund"] + npm_spec
subprocess.run(npm_command, env=env, check=True)

def inventory_tree(directory, scope):
    """Inventory every entry without following or undercounting symlink targets."""
    entries = []
    resolved_root = directory.resolve()

    def visit(path):
        metadata = path.lstat()
        relative = path.relative_to(directory)
        name = scope if not relative.parts else scope + "/" + relative.as_posix()
        row = {"path": name, "mode": format(stat.S_IMODE(metadata.st_mode), "04o")}
        if stat.S_ISLNK(metadata.st_mode):
            target = os.readlink(path)
            if not path.resolve(strict=True).is_relative_to(resolved_root):
                raise RuntimeError("Installed symlink escapes counted footprint: " + name)
            payload = os.fsencode(target)
            row.update(kind="symlink", bytes=len(payload), sha256=hashlib.sha256(payload).hexdigest(),
                       link_target=target, target_inside_counted_root=True)
        elif stat.S_ISREG(metadata.st_mode):
            payload = path.read_bytes()
            after = path.lstat()
            if (metadata.st_ino, metadata.st_size, metadata.st_mtime_ns) != (after.st_ino, after.st_size, after.st_mtime_ns):
                raise RuntimeError("Installed file changed during inventory: " + name)
            row.update(kind="file", bytes=len(payload), sha256=hashlib.sha256(payload).hexdigest())
        elif stat.S_ISDIR(metadata.st_mode):
            row.update(kind="directory", bytes=0, sha256=None)
        else:
            raise RuntimeError("Unsupported entry in installed footprint: " + name)
        entries.append(row)
        if row["kind"] == "directory":
            for child in sorted(path.iterdir()):
                visit(child)

    visit(directory)
    return entries


def initial_integration(project, runtime, binary):
    """Require the real initial MCP registration and skill, not empty placeholders."""
    home = Path(runtime["CODEX_HOME"])
    config = home / "config.toml"
    skill = home / "skills/kineti-gate/SKILL.md"
    manifest_path = project / ".kineti/install.json"
    manifest = json.loads(manifest_path.read_text())
    text = config.read_text()
    assert text.count("[mcp_servers.kineti]") == 1
    assert manifest["block"] in text
    assert Path(manifest["codex_home"]).resolve() == home.resolve()
    command = next(json.loads(line.removeprefix("command = ")) for line in text.splitlines() if line.startswith("command = "))
    args = next(json.loads(line.removeprefix("args = ")) for line in text.splitlines() if line.startswith("args = "))
    assert Path(command).resolve() == binary.resolve()
    assert args == ["mcp", "--root", str(project)]
    assert b"name: kineti-gate" in skill.read_bytes()
    assert manifest["skill_sha256"] == hashlib.sha256(skill.read_bytes()).hexdigest()
    return {"configuration": config, "skill": skill, "ownership_manifest": manifest_path,
            "state": project / ".kineti/state.json", "gate_configuration": project / ".kineti/gate.json"}

report = {"target": options.target, "version": VERSION, "commit": os.environ.get("GITHUB_SHA"), "run_id": os.environ.get("GITHUB_RUN_ID"),
          "platform": os.uname().sysname, "architecture": os.uname().machine, "outside_source_checkout": root not in work.parents and root != work,
          "downloads": {p.name: {"bytes": p.stat().st_size, "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in [crate, archive]},
          "installation_source": "published registry names" if options.published_registry else "local reviewed candidate archives",
          "digest_inputs_prove_human_identity": False, "publication_authority": False,
          "install_commands": {"cargo": cargo_command, "npm": npm_command},
          "routes": {}}
assert (report["platform"], report["architecture"]) == expected_hosts[options.target]
if options.published_registry:
    cache = Path(env.get("CARGO_HOME", str(Path.home() / ".cargo")))
    installed_crates = list((cache / "registry/cache").glob("*/kineti-cli-" + VERSION + ".crate"))
    if not installed_crates or any(hashlib.sha256(path.read_bytes()).hexdigest() != options.expected_cargo_sha256 for path in installed_crates):
        raise RuntimeError("Cargo installed registry cache does not match the exact reviewed crate")
    report["cargo_registry_cached_crate_verified"] = True
with tarfile.open(archive) as packed:
    checksum_manifest = json.load(packed.extractfile("package/checksums.json"))
    assert checksum_manifest["version"] == VERSION
    selected = packed.extractfile("package/native/" + options.target + "/kineti").read()
    expected_binary_sha = hashlib.sha256(selected).hexdigest()
    assert expected_binary_sha == checksum_manifest["targets"][options.target]
    assert hashlib.sha256((npm_root / "bin/kineti").read_bytes()).hexdigest() == expected_binary_sha
    installed_package = npm_root / "lib/node_modules/kineti"
    for member in packed.getmembers():
        relative = PurePosixPath(member.name)
        if member.isfile() and relative.parts[0] == "package" and not member.name.startswith("package/native/") and member.name != "package/bin/kineti":
            actual = installed_package.joinpath(*relative.parts[1:]).read_bytes()
            assert hashlib.sha256(actual).digest() == hashlib.sha256(packed.extractfile(member).read()).digest()
    report["npm_installed_payload_and_assets_match_reviewed_archive"] = True
evidence = work / "acceptance-evidence"
evidence.mkdir()
for route, installation in [("cargo", cargo_root), ("npm", npm_root)]:
    run = work / (route + "-runtime")
    project = run / "project"
    project.mkdir(parents=True)
    empty_path = run / "empty-path"
    empty_path.mkdir()
    runtime = {"PATH": str(empty_path), "HOME": str(run / "home"), "CODEX_HOME": str(run / "home/.codex"), "TZ": "UTC"}
    binary = installation / "bin/kineti"
    assert not shutil.which("node", path=runtime["PATH"])
    assert not shutil.which("bun", path=runtime["PATH"])
    observed = subprocess.run([str(binary), "--version"], env=runtime, cwd=project, capture_output=True, text=True)
    version = observed.stdout
    (evidence / (route + "-version.stdout")).write_text(version)
    (evidence / (route + "-version.stderr")).write_text(observed.stderr)
    observed.check_returncode()
    assert json.loads(version)["result"]["version"] == "0.6.1"
    observed = subprocess.run([str(binary), "self-test"], env=runtime, cwd=project, capture_output=True, text=True)
    tested = observed.stdout
    (evidence / (route + "-self-test.stdout")).write_text(tested)
    (evidence / (route + "-self-test.stderr")).write_text(observed.stderr)
    observed.check_returncode()
    for outcome in ["stale_release_blocked", "fresh_release_passed", "invalid_signature_rejected", "fabricated_output_accepted"]:
        assert json.loads(tested)["result"][outcome] is True
    initialized = subprocess.run([str(binary), "init", "--goal", "Disposable candidate install acceptance", "--codex-home", runtime["CODEX_HOME"], "--apply"], env=runtime, cwd=project, capture_output=True)
    (evidence / (route + "-init.stdout")).write_bytes(initialized.stdout)
    (evidence / (route + "-init.stderr")).write_bytes(initialized.stderr)
    initialized.check_returncode()
    integration = initial_integration(project, runtime, binary)
    registry = json.loads((root / "server.json").read_text())
    mcp_args = [arg["value"] for arg in registry["packages"][0]["packageArguments"]]
    messages = [{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "candidate-acceptance", "version": "0.6.1"}}},
                {"jsonrpc": "2.0", "method": "notifications/initialized"},
                {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
                {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "kineti_status", "arguments": {}}}]
    mcp = subprocess.run([str(binary)] + mcp_args, input="".join(json.dumps(m) + "\n" for m in messages), env=runtime, cwd=project, capture_output=True, text=True, timeout=30)
    (evidence / (route + "-mcp.stdout")).write_text(mcp.stdout)
    (evidence / (route + "-mcp.stderr")).write_text(mcp.stderr)
    mcp.check_returncode()
    replies = {r["id"]: r for r in map(json.loads, mcp.stdout.splitlines()) if "id" in r}
    assert len(replies[2]["result"]["tools"]) == 8
    assert not replies[3]["result"].get("isError", False)
    assert "Disposable candidate install acceptance" in json.dumps(replies[3])
    entries = inventory_tree(installation, "installation") + inventory_tree(run, "runtime")
    size = sum(entry["bytes"] for entry in entries)
    if size >= 10_000_000:
        raise RuntimeError(f"{options.target}/{route} installed footprint {size} blocks release")
    if route == "npm":
        assert not (installation / "lib/node_modules/kineti/native").exists()
    initial_files = {name: "runtime/" + path.relative_to(run).as_posix() for name, path in integration.items()}
    assert all(any(entry["path"] == relative and entry["kind"] == "file" for entry in entries) for relative in initial_files.values())
    inventory = {"schema": "kineti-installed-inventory-v1", "route": route, "target": options.target, "version": VERSION,
                 "commit": os.environ.get("GITHUB_SHA"), "run_id": os.environ.get("GITHUB_RUN_ID"),
                 "entries": entries, "installed_bytes": size,
                 "file_bytes": sum(entry["bytes"] for entry in entries if entry["kind"] == "file"),
                 "symlink_bytes": sum(entry["bytes"] for entry in entries if entry["kind"] == "symlink"),
                 "included": ["installation: binary and installed package assets", "runtime: initial state and Codex integration files"],
                 "excluded": ["Rust/Cargo toolchains and build cache", "npm cache", "extracted package sources", "acceptance transcripts and reports"],
                 "initial_integration_files": initial_files}
    inventory_path = work / (route + "-installed-inventory.json")
    inventory_path.write_text(json.dumps(inventory, indent=2) + "\n")
    report["routes"][route] = {"installed_bytes": size, "version": json.loads(version), "self_test": json.loads(tested),
                                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "node_bun_absent_from_runtime_path": True, "registry_mcp_discovery_and_call": True,
                                "initial_integration_verified_and_counted": True, "inventory_file": inventory_path.name,
                                "inventory_sha256": hashlib.sha256(inventory_path.read_bytes()).hexdigest(), "inventory_entry_count": len(entries)}

disabled = work / "npm-scripts-disabled"
subprocess.run(["npm", "install", "--ignore-scripts", "--global", "--prefix", str(disabled), "--no-audit", "--no-fund"] + npm_spec, env=env, check=True)
p = subprocess.run([str(disabled / "bin/kineti"), "--version"], capture_output=True, text=True)
assert p.returncode == 2 and "scripts" in p.stderr
report["scripts_disabled_error_verified"] = True
report["npm_unpacked_bytes"] = sum(m.size for m in tarfile.open(archive).getmembers() if m.isfile())
report["cargo_unpacked_bytes"] = sum(m.size for m in tarfile.open(crate).getmembers() if m.isfile())
(work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))

"""Actual native candidate installation checks. All outputs stay in the project."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
from pathlib import PurePosixPath
import subprocess
import tarfile

parser = argparse.ArgumentParser()
parser.add_argument("--artifacts", required=True)
parser.add_argument("--target", required=True)
parser.add_argument("--rust-target", required=True)
options = parser.parse_args()
root = Path(__file__).resolve().parents[1]
artifacts = Path(options.artifacts).resolve()
for name, expected in json.loads((artifacts / "SHA256SUMS.json").read_text()).items():
    if hashlib.sha256((artifacts / name).read_bytes()).hexdigest() != expected:
        raise RuntimeError("Candidate artifact digest mismatch: " + name)
work = root / ".kineti/install-acceptance" / options.target
if work.exists():
    raise RuntimeError("Acceptance directory already exists; preserve and inspect before rerun")
work.mkdir(parents=True)
source = work / "source"
source.mkdir()
crate = artifacts / "kineti-cli-0.6.0.crate"
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
crate_source = source / "kineti-cli-0.6.0"
env = dict(os.environ, CARGO_TARGET_DIR=str(work / "cargo-target"), npm_config_cache=str(work / "npm-cache"))
cargo_root = work / "cargo-install"
subprocess.run(["cargo", "install", "--path", str(crate_source), "--locked", "--target", options.rust_target, "--root", str(cargo_root)], env=env, check=True)
npm_root = work / "npm-install"
archive = artifacts / "kineti-0.6.0.tgz"
subprocess.run(["npm", "install", "--global", "--prefix", str(npm_root), "--no-audit", "--no-fund", str(archive)], env=env, check=True)

def footprint(directory):
    total = 0
    for path in directory.rglob("*"):
        if path.is_symlink():
            total += len(os.readlink(path).encode())
        elif path.is_file():
            total += path.stat().st_size
    return total

report = {"target": options.target, "platform": os.uname().sysname, "architecture": os.uname().machine,
          "downloads": {p.name: {"bytes": p.stat().st_size, "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in [crate, archive]},
          "routes": {}}
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
    version = subprocess.check_output([str(binary), "--version"], env=runtime, cwd=project, text=True)
    assert json.loads(version)["result"]["version"] == "0.6.0"
    tested = subprocess.check_output([str(binary), "self-test"], env=runtime, cwd=project, text=True)
    assert json.loads(tested)["result"]["fabricated_output_accepted"] is True
    subprocess.run([str(binary), "init", "--goal", "Disposable candidate install acceptance", "--codex-home", runtime["CODEX_HOME"], "--apply"], env=runtime, cwd=project, check=True, stdout=subprocess.PIPE)
    registry = json.loads((root / "server.json").read_text())
    mcp_args = [arg["value"] for arg in registry["packages"][0]["packageArguments"]]
    messages = [{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "candidate-acceptance", "version": "0.6.0"}}},
                {"jsonrpc": "2.0", "method": "notifications/initialized"},
                {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
                {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "kineti_status", "arguments": {}}}]
    mcp = subprocess.run([str(binary)] + mcp_args, input="".join(json.dumps(m) + "\n" for m in messages), env=runtime, cwd=project, capture_output=True, text=True, timeout=30, check=True)
    replies = {r["id"]: r for r in map(json.loads, mcp.stdout.splitlines()) if "id" in r}
    assert len(replies[2]["result"]["tools"]) == 8
    assert not replies[3]["result"].get("isError", False)
    assert "Disposable candidate install acceptance" in json.dumps(replies[3])
    size = footprint(installation) + footprint(run)
    if size >= 10_000_000:
        raise RuntimeError(f"{options.target}/{route} installed footprint {size} blocks release")
    if route == "npm":
        assert not (installation / "lib/node_modules/kineti/native").exists()
    report["routes"][route] = {"installed_bytes": size, "version": json.loads(version), "self_test": json.loads(tested),
                                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "node_bun_absent_from_runtime_path": True, "registry_mcp_discovery_and_call": True}

disabled = work / "npm-scripts-disabled"
subprocess.run(["npm", "install", "--ignore-scripts", "--global", "--prefix", str(disabled), "--no-audit", "--no-fund", str(archive)], env=env, check=True)
p = subprocess.run([str(disabled / "bin/kineti"), "--version"], capture_output=True, text=True)
assert p.returncode == 2 and "scripts" in p.stderr
report["scripts_disabled_error_verified"] = True
report["npm_unpacked_bytes"] = sum(m.size for m in tarfile.open(archive).getmembers() if m.isfile())
report["cargo_unpacked_bytes"] = sum(m.size for m in tarfile.open(crate).getmembers() if m.isfile())
(work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps(report, indent=2))

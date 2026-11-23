#!/usr/bin/env python3
"""Package one native binary, extract the exact archive, and exercise Path A installed CLI and MCP workflows."""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import stat
import subprocess
import tarfile


def sha(data):
    return hashlib.sha256(data).hexdigest()


def run(args, **kwargs):
    return subprocess.check_output(args, text=True, **kwargs)


def collect_files(workspace, binary, cargo_home, triple):
    files = {"kineti": binary, "LICENSE": workspace / "LICENSE"}
    argv = [
        "cargo",
        "tree",
        "--locked",
        "--offline",
        "--target",
        triple,
        "--manifest-path",
        str(workspace / "core-native/Cargo.toml"),
        "--prefix",
        "none",
        "--format",
        "{p}",
        "--edges",
        "normal,build",
    ]
    deps = run(argv, env=dict(os.environ, CARGO_HOME=str(cargo_home)))
    registries = list((cargo_home / "registry/src").iterdir())
    for line in sorted(set(deps.splitlines())):
        name, version = line.split()[:2]
        if name == "kineti-cli":
            continue
        crate_name = name + "-" + version.removeprefix("v")
        roots = [
            directory / crate_name
            for directory in registries
            if (directory / crate_name).is_dir()
        ]
        assert len(roots) == 1, crate_name
        crate = roots[0]
        notices = [
            p
            for p in crate.rglob("*")
            if p.is_file()
            and p.name.lower().startswith(
                ("license", "copying", "notice", "copyright")
            )
        ]
        assert notices, f"Missing notices: {crate}"
        for p in notices:
            assert not p.is_symlink()
            files[f"notices/{crate_name}/{p.relative_to(crate).as_posix()}"] = p
    docs = Path(run(["rustc", "--print", "sysroot"]).strip()) / "share/doc/rust"
    copyright_lib = docs / "COPYRIGHT-library.html"
    if copyright_lib.is_file():
        files["notices/rust/COPYRIGHT-library.html"] = copyright_lib
    licenses_dir = docs / "licenses"
    if licenses_dir.is_dir():
        licenses = sorted(licenses_dir.glob("*.txt"))
        assert licenses, "Missing Rust license documents"
        for p in licenses:
            files["notices/rust/licenses/" + p.name] = p
    return files


def archive_bytes(files):
    buf = io.BytesIO()
    members = {}
    with gzip.GzipFile(fileobj=buf, mode="wb", mtime=0, filename="") as gz:
        with tarfile.open(fileobj=gz, mode="w", format=tarfile.USTAR_FORMAT) as tar:
            for name, source in sorted(files.items()):
                data = source.read_bytes()
                member = tarfile.TarInfo(name)
                member.size = len(data)
                member.mode = 0o755 if name == "kineti" else 0o644
                tar.addfile(member, io.BytesIO(data))
                members[name] = {
                    "bytes": len(data),
                    "mode": member.mode,
                    "sha256": sha(data),
                }
    return buf.getvalue(), members


def main():
    parser = argparse.ArgumentParser()
    for key in ("source", "binary", "cargo-home", "output"):
        parser.add_argument("--" + key, type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--preview", action="store_true")
    args = parser.parse_args()

    host = (platform.system(), platform.machine())
    targets = {
        "aarch64-apple-darwin": ("Darwin", "arm64"),
        "x86_64-apple-darwin": ("Darwin", "x86_64"),
        "x86_64-unknown-linux-gnu": ("Linux", "x86_64"),
        "x86_64-unknown-linux-musl": ("Linux", "x86_64"),
        "aarch64-unknown-linux-gnu": ("Linux", "aarch64"),
        "aarch64-unknown-linux-musl": ("Linux", "aarch64"),
    }
    assert targets[args.target] == host, (args.target, host)

    binary = args.binary.resolve().read_bytes()
    for path in (Path.home(), args.source.resolve(), args.cargo_home.resolve()):
        assert os.fsencode(str(path)) not in binary, (
            f"Absolute build path in binary: {path}"
        )

    files = collect_files(
        args.source.resolve(),
        args.binary.resolve(),
        args.cargo_home.resolve(),
        args.target,
    )
    contents, members = archive_bytes(files)
    proof = {
        "target": args.target,
        "host": platform.platform(),
        "rustc": run(["rustc", "-Vv"]),
        "binary": sha(binary),
        "binary_bytes": len(binary),
        "archive_sha256": sha(contents),
        "download_bytes": len(contents),
        "unpacked_bytes": sum(m["bytes"] for m in members.values()),
        "members": members,
    }
    if args.preview:
        print(json.dumps(proof))
        return

    out = args.output.resolve()
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    archive = out / ("kineti-" + args.target + ".tar.gz")
    archive.write_bytes(contents)

    stage, home, project, empty, temp = [
        out / f for f in ("installation", "home", "project", "empty-path", "tmp")
    ]
    for path in (stage, home, project, empty, temp):
        path.mkdir(mode=0o700)

    with tarfile.open(archive) as tar:
        assert tar.getnames() == sorted(members)
        for m in tar:
            assert m.isfile() and m.name in members and m.mode == members[m.name]["mode"]
            data = tar.extractfile(m).read()
            assert sha(data) == members[m.name]["sha256"]
            dest = stage / m.name
            dest.parent.mkdir(parents=True, exist_ok=True)
            with dest.open("xb") as f:
                f.write(data)
            dest.chmod(m.mode)

    runtime = dict(
        PATH=str(empty),
        HOME=str(home),
        CODEX_HOME=str(home / ".codex"),
        TMPDIR=str(temp),
        TZ="UTC",
    )

    def raw_call(argv, cwd, wire=None):
        cmd = [str(stage / "kineti"), *argv]
        p = subprocess.run(
            cmd,
            cwd=cwd,
            env=runtime,
            input=wire,
            capture_output=True,
            text=True,
            timeout=60,
        )
        with (out / "commands.jsonl").open("a") as f:
            f.write(
                json.dumps(
                    dict(
                        argv=cmd,
                        cwd=str(cwd),
                        env=runtime,
                        stdin=wire,
                        stdout=p.stdout,
                        stderr=p.stderr,
                        exit=p.returncode,
                    )
                )
                + "\n"
            )
        return p

    def cli(argv, cwd=project, expect_code=0):
        p = raw_call(argv, cwd)
        assert p.returncode == expect_code, (argv, p.returncode, p.stdout, p.stderr)
        return json.loads(p.stdout.splitlines()[0])

    def mcp_call(name, arguments, cwd=project):
        requests = [
            {"method": "initialize", "params": {"protocolVersion": "2025-03-26"}},
            {"method": "tools/call", "params": {"name": name, "arguments": arguments}},
        ]
        wire = "".join(
            json.dumps(dict(jsonrpc="2.0", id=i + 1, **row)) + "\n"
            for i, row in enumerate(requests)
        )
        p = raw_call(["mcp"], cwd, wire)
        assert p.returncode == 0, (p.returncode, p.stdout, p.stderr)
        return json.loads(p.stdout.splitlines()[-1])["result"]

    # 1. Version & self-test
    version = cli(["--version"])["result"]
    selftest = cli(["self-test"])["result"]
    assert all(
        selftest[k] is True
        for k in (
            "fresh_release_passed",
            "stale_release_blocked",
            "invalid_signature_rejected",
            "fabricated_output_accepted",
        )
    )

    # 2. Init & project-only status (with empty PATH, proving Tier 2 pure-Rust walker!)
    init_cmd = [
        "init",
        "--goal",
        "Isolated Path A release archive check",
        "--codex-home",
        runtime["CODEX_HOME"],
    ]
    cli(init_cmd)
    cli(init_cmd + ["--apply"])
    status = cli(["status", "--project-only"])["result"]
    mcp_status = mcp_call("kineti_status", {"project_only": True})
    assert mcp_status.get("isError") is False
    assert status == json.loads(mcp_status["content"][0]["text"])
    assert status["state"]["root_goal"] == init_cmd[2]

    # 3. Tier 2 empty-PATH polyglot exit-code run, 1-byte src/legacy/ drift detection, fail-closed CI
    (project / "src" / "legacy").mkdir(parents=True)
    (project / "src" / "legacy" / "auth.txt").write_text("allow=false\n")
    (project / "tests").mkdir()
    (project / "tests" / "check.txt").write_text("required test fixture\n")

    allowlisted_cmd = ["/bin/sh", "-c", "exit 0"]
    cli(
        [
            "test",
            "run",
            "--label",
            "unit",
            "--parser",
            "exit-code",
            "--required-test",
            "tests/check.txt",
            "--",
            *allowlisted_cmd,
        ]
    )
    # Fail-closed CI when commands map is empty
    cli(["ci", "--require-evidence", "unit"], expect_code=1)

    # Configure commands allowlist in .kineti/gate.json
    gate_path = project / ".kineti" / "gate.json"
    gate_cfg = json.loads(gate_path.read_text())
    gate_cfg["required_checks"] = ["unit"]
    gate_cfg["commands"] = {"unit": allowlisted_cmd}
    gate_path.write_text(json.dumps(gate_cfg, indent=2) + "\n")

    # Re-record after updating .kineti/gate.json (it lives in .kineti so inventory unchanged, check passes!)
    cli(["ci", "--require-evidence", "unit"], expect_code=0)

    # Mutate 1 byte inside src/legacy/ -> MUST return exit 4 (stale!)
    (project / "src" / "legacy" / "auth.txt").write_text("allow=true!\n")
    cli(["test", "check", "--label", "unit"], expect_code=4)

    # 4. MCP allowlist pass, non-allowlisted RCE refusal, removed saga tool refusal
    mcp_ok = mcp_call(
        "kineti_evidence_record",
        {
            "label": "unit",
            "command": allowlisted_cmd,
            "required_tests": ["tests/check.txt"],
            "parser": "exit-code",
        },
    )
    assert mcp_ok.get("isError") is False, mcp_ok

    mcp_rce = mcp_call(
        "kineti_evidence_record",
        {
            "label": "unit",
            "command": ["/bin/sh", "-c", "echo pwned"],
            "required_tests": ["tests/check.txt"],
            "parser": "exit-code",
        },
    )
    assert mcp_rce.get("isError") is True, mcp_rce

    mcp_saga = mcp_call(
        "kineti_saga_push",
        {"label": "x", "command": ["/bin/sh", "-c", "echo pwned"]},
    )
    assert mcp_saga.get("isError") is True, mcp_saga

    # 5. Footprint & integrity inventory (< 10,000,000 bytes)
    inventory = {}
    for base in (stage, home, project):
        for path in base.rglob("*"):
            assert not path.is_symlink()
            if path.is_file():
                inventory[path.relative_to(out).as_posix()] = {
                    "bytes": path.stat().st_size,
                    "sha256": sha(path.read_bytes()),
                    "mode": stat.S_IMODE(path.stat().st_mode),
                }
    total = sum(item["bytes"] for item in inventory.values())
    assert total < 10_000_000 and sha(archive.read_bytes()) == proof["archive_sha256"]

    proof.update(
        archive=archive.name,
        version=version,
        installed_bytes=total,
        limit_exclusive=10_000_000,
        files=inventory,
        commands_sha256=sha((out / "commands.jsonl").read_bytes()),
        self_test=selftest,
        cli_mcp_project_status_equal=True,
        tier2_empty_path_legacy_drift_blocked=True,
        mcp_allowlist_and_saga_removal_verified=True,
    )
    (out / "installation.json").write_text(json.dumps(proof, indent=2) + "\n")
    print(args.target, proof["archive_sha256"], total, "installed bytes")


if __name__ == "__main__":
    main()

"""Exercise the installed app-server MCP API without credentials or a model turn.

Uses init-generated gate integration with --gate, or an early native fixture. All mutable
state is isolated under .kineti/gate-060-feasibility. No API key is inherited.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import shutil
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / ".kineti/gate-060-feasibility"
EXPECTED = "codex-cli 0.163.0-alpha.11.2"


def main():
    global STATE, EXPECTED
    parser = argparse.ArgumentParser()
    parser.add_argument("--codex")
    parser.add_argument("--expected-version", default=EXPECTED)
    parser.add_argument("--gate")
    parser.add_argument("--state-root")
    options = parser.parse_args()
    EXPECTED = options.expected_version
    if options.state_root:
        STATE = Path(options.state_root).resolve()
        if not STATE.is_relative_to(ROOT / ".kineti"):
            raise RuntimeError("Probe state must stay inside project .kineti")
    STATE.mkdir(parents=True, exist_ok=True)
    # Capture the installed binary before switching HOME/PATH.
    codex = Path(options.codex or shutil.which("codex")).resolve()
    version = subprocess.check_output([str(codex), "--version"], text=True).strip()
    if version != EXPECTED:
        raise RuntimeError(f"Pinned Codex required: {EXPECTED}; found {version}")
    if options.gate:
        native = Path(options.gate).resolve()
    else:
        fixture = ROOT / "tests/fixtures/gate-060-feasibility/Cargo.toml"
        build_env = dict(os.environ, CARGO_HOME=str(ROOT / ".kineti/cargo-home"))
        subprocess.run(["cargo", "build", "--offline", "--manifest-path", str(fixture),
                        "--target-dir", str(STATE / "native-target")],
                       check=True, cwd=ROOT, env=build_env)
        native = STATE / "native-target/debug/kineti-native-mcp-feasibility"
    for directory in ["home", "codex-home", "tmp", "work"]:
        (STATE / directory).mkdir(exist_ok=True)
    config_path = STATE / "codex-home/config.toml"
    config_path.write_text(
        'model = "gpt-6.1-sol"\nmodel_provider = "feasibility-offline"\n'
        '[model_providers.feasibility-offline]\nname = "No model calls permitted"\n'
        'base_url = "http://127.0.0.1:9/v1"\nwire_api = "responses"\n'
        'requires_openai_auth = false\n'
        '[analytics]\nenabled = false\n[feedback]\nenabled = false\n'
        + ('' if options.gate else '[mcp_servers.kineti_feasibility]\n'
           f'command = {json.dumps(str(native))}\nstartup_timeout_sec = 15\n')
    )
    env = {"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "HOME": str(STATE / "home"),
           "CODEX_HOME": str(STATE / "codex-home"), "TMPDIR": str(STATE / "tmp"),
           "XDG_CONFIG_HOME": str(STATE / "home/config"),
           "XDG_DATA_HOME": str(STATE / "home/data"),
           "XDG_CACHE_HOME": str(STATE / "home/cache")}
    server = "kineti_feasibility"
    tool = "kineti_feasibility_status"
    if options.gate:
        subprocess.run([str(native), "init", "--goal", "Disposable native host acceptance", "--codex-home", str(STATE / "codex-home"), "--apply"],
                       cwd=STATE / "work", env=env, check=True, capture_output=True)
        server = "kineti"
        tool = "kineti_status"
    transcript = []
    received = queue.Queue()
    stderr_path = STATE / "codex-probe.stderr.log"
    with stderr_path.open("w") as err:
        process = subprocess.Popen([str(codex), "app-server", "--stdio"],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=err, text=True, cwd=STATE / "work", env=env)
        def read_stdout():
            for line in process.stdout:
                received.put(line)
            received.put(None)
        reader = threading.Thread(target=read_stdout, daemon=True)
        reader.start()
        next_id = 0

        def send(method, params, notify=False):
            nonlocal next_id
            # Hard allowlist: this probe cannot submit a model turn.
            assert method in {"initialize", "initialized", "config/read", "thread/start",
                              "mcpServerStatus/list", "mcpServer/tool/call"}
            message = {"method": method, "params": params}
            if not notify:
                next_id += 1
                message["id"] = next_id
            transcript.append({"direction": "send", "message": message})
            process.stdin.write(json.dumps(message) + "\n")
            process.stdin.flush()
            if notify:
                return None
            deadline = time.monotonic() + 40
            while time.monotonic() < deadline:
                line = received.get(timeout=max(0.1, deadline - time.monotonic()))
                if line is None:
                    raise RuntimeError(f"app-server exited: {stderr_path}")
                response = json.loads(line)
                transcript.append({"direction": "receive", "message": response})
                if response.get("id") == message["id"]:
                    if "error" in response:
                        raise RuntimeError(json.dumps(response))
                    return response["result"]
            raise TimeoutError(method)
        result = {"version": version, "codex_sha256": hashlib.sha256(codex.read_bytes()).hexdigest(),
                  "native_sha256": hashlib.sha256(native.read_bytes()).hexdigest(),
                  "config_sha256": hashlib.sha256(config_path.read_bytes()).hexdigest(),
                  "fixture_only": not bool(options.gate), "init_generated_registration": bool(options.gate), "model_turn_started": False, "passed": False}
        try:
            send("initialize", {"clientInfo": {"name": "kineti_gate_feasibility", "version": "0.6.0"},
                                "capabilities": {"experimentalApi": True}})
            send("initialized", {}, notify=True)
            config = send("config/read", {"includeLayers": True})
            assert server in json.dumps(config), "Generated configuration not loaded"
            thread = send("thread/start", {"cwd": str(STATE / "work"), "ephemeral": True,
                                          "approvalPolicy": "never", "sandbox": "read-only"})
            thread_id = thread["thread"]["id"]
            discovery = send("mcpServerStatus/list", {"threadId": thread_id,
                             "serverName": server, "detail": "toolsAndAuthOnly"})
            assert tool in json.dumps(discovery), "Native tool not discovered"
            call = send("mcpServer/tool/call", {"threadId": thread_id, "server": server,
                        "tool": tool, "arguments": {}})
            assert not call.get("isError", False), "Native MCP call failed"
            if not options.gate:
                assert "kineti-native-feasibility-ok" in json.dumps(call), "Native MCP result missing"
            else:
                assert "Disposable native host acceptance" in json.dumps(call), "Native project state missing"
            result.update(passed=True, discovery=discovery, call=call)
        except Exception as exc:
            result["error"] = str(exc)
            raise
        finally:
            process.stdin.close()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
            (STATE / "codex-probe.transcript.json").write_text(json.dumps(transcript, indent=2))
            (STATE / "codex-probe.result.json").write_text(json.dumps(result, indent=2))
    print("Native MCP configuration, discovery and real call passed; no model turn started.")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()

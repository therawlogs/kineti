"""Public runner identity for candidate acceptance, without credentials or home paths."""
import json
import os
import platform
import subprocess

def output(command):
    return subprocess.check_output(command, text=True).strip()

print(json.dumps({"commit": os.environ.get("GITHUB_SHA"), "run_id": os.environ.get("GITHUB_RUN_ID"),
    "runner_os": os.environ.get("RUNNER_OS"), "runner_arch": os.environ.get("RUNNER_ARCH"),
    "system": platform.system(), "release": platform.release(), "machine": platform.machine(),
    "os_version": output(["sw_vers"]) if platform.system() == "Darwin" else open("/etc/os-release").read(),
    "rustc": output(["rustc", "--version"]), "cargo": output(["cargo", "--version"])}, indent=2))

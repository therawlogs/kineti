"""Assemble one npm archive from the four reviewed native build artifacts."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--artifacts", required=True)
parser.add_argument("--output", required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
artifacts = Path(args.artifacts).resolve()
out = Path(args.output).resolve()
out.mkdir(parents=True, exist_ok=True)
sums = {"version": "0.6.1", "targets": {}}
for target in ["linux-x64", "linux-arm64", "darwin-x64", "darwin-arm64"]:
    source = artifacts / ("kineti-" + target)
    destination = root / "native" / target / "kineti"
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    destination.chmod(0o755)
    sums["targets"][target] = hashlib.sha256(source.read_bytes()).hexdigest()
(root / "checksums.json").write_text(json.dumps(sums, indent=2) + "\n")
subprocess.run(["node", "scripts/check-native-package.cjs"], cwd=root, check=True)
packed = subprocess.check_output(["npm", "pack", "--ignore-scripts", "--json", "--pack-destination", str(out)], cwd=root, text=True)
(out / "npm-pack.json").write_text(packed)
crate = artifacts / "kineti-cli-0.6.1.crate"
shutil.copyfile(crate, out / crate.name)
digests = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in out.iterdir() if p.suffix in [".tgz", ".crate"]}
(out / "SHA256SUMS.json").write_text(json.dumps(digests, indent=2) + "\n")
print(json.dumps({"digests": digests, "npm": json.loads(packed)}, indent=2))

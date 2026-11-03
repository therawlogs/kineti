"""Offline component acceptance of the installed protected macOS guest worker.

This exercises the native lease and fixed guest transport, not a substitute
repair engine. Passing never admits the production owner backend or A01–A14.
No installation, provisioning, provider request or credential operation occurs.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--binary', required=True)
p.add_argument('--binding', required=True)
p.add_argument('--worker-sha256', required=True)
p.add_argument('--sdk-sha256', required=True)
p.add_argument('--run-id', required=True)
p.add_argument('--output', required=True)
a = p.parse_args()
if sys.platform != 'darwin' or os.geteuid() != 501:
    p.error('requires the existing macOS host owner role; no platform fallback')
if not re.fullmatch('[a-z0-9-]{1,35}', a.run_id):
    p.error('run-id requires 1..35 lowercase ASCII letters, digits or hyphens')
for digest in [a.worker_sha256, a.sdk_sha256]:
    if not re.fullmatch('[0-9a-f]{64}', digest):
        p.error('artifact pins require lowercase SHA-256')
root = Path(__file__).resolve().parent.parent
out = Path(a.output).resolve()
if not out.is_relative_to(root / '.kineti') or out.exists():
    p.error('output must be a new project-local .kineti evidence directory')
out.mkdir(mode=0o700, parents=True)
# Native vm-run validates all paths, config, identity, executable and kernel
# ownership before creating the lifetime lease. This script never claims that
# parsing this JSON by itself establishes a VM boundary.
binding = Path(a.binding).resolve(strict=True)
v = json.loads(binding.read_text())
binary = Path(a.binary).resolve(strict=True)
env = os.environ.copy()
env['LIMA_HOME'] = v['lima_home']
ssh = ['/usr/bin/ssh', '-F', str(Path(v['instance_dir']) / 'ssh.config'),
       '-o', 'BatchMode=yes', '-o', 'ControlMaster=no', '-o', 'ControlPath=none',
       '-o', 'ConnectTimeout=3', 'lima-' + v['instance'],
       '/private/var/lib/kineti-owner-guest-v1/bin/kineti host guest-connect']
r = {'schema': 'kineti-cold-worker-component-acceptance-v1',
     'passed': False, 'production_owner_service_accepted': False,
     'new_paid_usd_micros': 0, 'provider_logins': 0, 'model_turns': 0,
     'native_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
     'binding_sha256': hashlib.sha256(binding.read_bytes()).hexdigest(),
     'commands': [], 'checks': {}}


def save():
    (out / 'result.json').write_text(json.dumps(r, indent=2))


def command(argv, payload=None, timeout=35):
    child = subprocess.run([str(x) for x in argv], env=env,
                           input=payload, capture_output=True, text=True,
                           timeout=timeout)
    r['commands'].append({'argv': [str(x) for x in argv],
                          'exit': child.returncode, 'stdout': child.stdout,
                          'stderr': child.stderr})
    save()
    return child


def rpc(method, payload=None, denied=False):
    q = {'schema': 'kineti-guest-control-v1', 'method': method, 'payload': payload}
    child = command(ssh, json.dumps(q) + '\n', 100)
    if denied:
        assert child.returncode == 3, child.stderr + child.stdout
        return
    assert child.returncode == 0, child.stderr + child.stdout
    return json.loads(child.stdout)['result']


def boot(name, action, orderly=False):
    child = None
    with (out / (name + '-supervisor.log')).open('w') as log:
        try:
            child = subprocess.Popen([str(binary), 'host', 'vm-run', '--binding',
                                      str(binding)], env=env, stdin=subprocess.PIPE,
                                     stdout=log, stderr=log)
            # Do not touch SSH configuration from an unvalidated binding.
            # The native ready event follows executable/config/VM validation.
            end = time.monotonic() + 100
            while time.monotonic() < end:
                assert child.poll() is None, 'native VM binding or startup refused'
                lines = (out / (name + '-supervisor.log')).read_text().splitlines()
                if any('"ready":true' in line and
                       '"schema":"kineti-vm-supervisor-v1"' in line for line in lines):
                    break
                time.sleep(.1)
            else:
                raise RuntimeError('native VM readiness deadline exceeded')
            end = time.monotonic() + 100
            while time.monotonic() < end:
                assert child.poll() is None, 'owner ended before guest readiness'
                ready = command(ssh, json.dumps({
                    'schema': 'kineti-guest-control-v1', 'method': 'probe',
                    'payload': None}) + '\n')
                if ready.returncode == 0:
                    probe = json.loads(ready.stdout)['result']
                    break
                time.sleep(.5)
            else:
                raise RuntimeError('protected worker readiness deadline exceeded')
            assert probe['worker_sha256'] == a.worker_sha256
            assert probe['sdk_sha256'] == a.sdk_sha256
            assert probe['sdk_version'] == 'codex-cli 0.164.0'
            assert probe['processes'] == 512 and probe['processes_per_uid'] == 128
            assert int(probe['memory_bytes']) == 4294967296
            assert probe['task_execution_admitted'] is False
            r[name + '_probe'] = probe
            action()
            if orderly:
                child.wait(timeout=100)
                assert child.returncode == 0, 'orderly guest shutdown failed'
                r['checks']['orderly_stop_without_forcing'] = True
        finally:
            if child is not None and child.poll() is None:
                child.stdin.close()
                child.wait(timeout=40)
            if child is not None:
                r[name + '_owner_exit'] = child.returncode
                # Never race a still-owned setup/foreground launcher.
                stopped = command([binary, 'host', 'vm-revoke', '--binding', binding])
                r[name + '_stopped_verified'] = stopped.returncode == 0
                save()
                assert stopped.returncode == 0, 'VM stop is uncertain; inspect retained work'


content = b'copied work survives production worker checkpoints\n'
h = hashlib.sha256(content).hexdigest()
q = {'schema': 'kineti-guest-cold-checkpoint-v1', 'id': a.run_id,
     'session': 'offline-checkpoint',
     'grant_digest': hashlib.sha256(b'offline synthetic checkpoint grant').hexdigest(),
     'expires_at': int(time.time()) + 1800,
     'checkpoint': {'schema': 'kineti-vm-checkpoint-v1',
                    'entries': {'value.txt': {'kind': 'file', 'mode': 420,
                                            'uid': 501, 'gid': 20,
                                            'bytes': len(content), 'sha256': h}},
                    'blobs': {h: base64.b64encode(content).decode()}}}


def prepare():
    rpc('exec', {'argv': ['/usr/bin/true']}, denied=True)
    rpc('probe', {'argv': ['/usr/bin/true']}, denied=True)
    r['checks']['no_arbitrary_root_command'] = True
    r['prepared'] = rpc('cold-prepare', q)
    rpc('cold-prepare', q, denied=True)
    r['checks']['duplicate_prepare_preserves_partial_work'] = True
    q['checkpoint'] = None
    rpc('cold-export', q, denied=True)
    r['checks']['live_export_denied'] = True
    forged = dict(q, grant_digest='0' * 64)
    rpc('cold-shutdown', forged, denied=True)
    expired = dict(q, expires_at=1)
    rpc('cold-shutdown', expired, denied=True)
    r['checks']['stale_and_expired_requests_denied'] = True
    r['shutdown'] = rpc('cold-shutdown', q)
    assert r['shutdown']['stopped_verified'] is False


def export():
    wire = rpc('cold-export', q)
    assert wire['read_only_verified'] is True
    assert wire['previous_boot'] == r['prepared']['boot']
    assert wire['previous_boot'] != wire['export_boot']
    assert base64.b64decode(wire['checkpoint']['blobs'][h]) == content
    r['exported'] = wire
    rpc('cold-export', q, denied=True)
    r['checks']['fresh_readonly_export_and_replay_denial'] = True


try:
    boot('prepare', prepare, orderly=True)
    boot('export', export)
    r['passed'] = True
finally:
    save()
print(json.dumps({'component_passed': r['passed'],
                  'production_owner_service_accepted': False,
                  'evidence': str(out / 'result.json')}))

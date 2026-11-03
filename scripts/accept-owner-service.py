"""Offline acceptance of the real owner-service CLI and pinned app-server.

Run only in the disposable, preconfigured Linux acceptance VM as its owner.
No provider login or model-turn API is exposed by this script.
"""
import argparse,os,json,subprocess,hashlib,time,pathlib,pty,select,sys,re,shlex
parser=argparse.ArgumentParser()
parser.add_argument('--binary',required=True)
parser.add_argument('--sdk',required=True)
parser.add_argument('--run-id',required=True)
parser.add_argument('--task-user',default='kineti-test-one')
parser.add_argument('--task-uid',type=int,default=2501)
a=parser.parse_args()
if sys.platform!='linux' or os.geteuid()!=0:raise SystemExit('BLOCKED: use the configured independent Linux owner role; no platform fallback')
if not re.fullmatch('[a-z0-9-]{1,35}',a.run_id):parser.error('run-id requires lowercase letters, digits and hyphens')
if not re.fullmatch('[a-z0-9-]{1,48}',a.task_user) or a.task_uid<=0:parser.error('existing task identity required')
P=pathlib.Path
base=P('/var/lib/kineti-owner-native-v1');project=P('/var/lib/kineti-owner-acceptance-'+a.run_id)
assert base.exists() and not project.exists()
binary=P(a.binary).resolve()
assert binary.is_file()
preserved={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in base.rglob('*') if p.is_file() and not p.is_symlink()}
project.mkdir(mode=0o755)
def cmd(args,cwd=project):
 p=subprocess.run([str(binary),*args],cwd=cwd,capture_output=True,text=True);return {'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr}
assert cmd(['init','--goal','Disposable owner service fixture','--apply'])['exit']==0
(project/'value.txt').write_text('broken\n');(project/'untracked.txt').write_text('preserve original\n')
state=json.loads((project/'.kineti/state.json').read_text())
identity=hashlib.sha256((str(project)+'\0'+state['root_goal']+'\0'+state['root_goal_locked_at']).encode()).hexdigest()
host=P(a.sdk).resolve()
g={'schema':'kineti-task-request-v1','id':a.run_id,'session':'session-v1','project_root':str(project),'project_identity':identity,'purpose':'Fixture failing check, exact local repair, fresh passing check','task_uid':a.task_uid,'task_user':a.task_user,'host':{'path':str(host),'sha256':hashlib.sha256(host.read_bytes()).hexdigest(),'version':'0.166.0'},'checks':[{'label':'value','kind':'test','argv':['/bin/sh','-c',"if grep -qx fixed value.txt; then printf '1 pass\n0 fail\n'; else printf '0 pass\n1 fail\n'; exit 1; fi"],'inputs':['.']}],'repair_argv':['/bin/sh','-c',"printf 'fixed\n' > value.txt"],'exclusions':[],'runtime_inputs':['/bin/busybox'],'limits':{'expires_at':int(time.time())+900,'wall_seconds':600,'command_seconds':20,'repair_attempts':1,'verification_attempts':3,'snapshot_bytes':10000000,'concurrency':1,'new_paid_usd_micros':0,'memory_bytes':1073741824,'pids':128},'network_destinations':[],'credential_handles':[],'resource_class':'owned-local','automatic_rollback':False}
request=project/'.kineti/owner-acceptance/request.json';request.parent.mkdir(mode=0o700);request.write_text(json.dumps(g))
# Scripted PTY is a fixture of the owner role, not human identity acceptance.
master,slave=pty.openpty();p=subprocess.Popen([str(binary),'task','approve','--owner-store',str(base),'--request',str(request)],stdin=slave,stdout=slave,stderr=slave,cwd=project);os.close(slave);out=b'';sent=False
while p.poll() is None:
 if select.select([master],[],[],1)[0]:
  try:out+=os.read(master,65536)
  except OSError:break
 if b'2. Deny' in out and not sent:os.write(master,b'1\n');sent=True
p.wait(timeout=10);os.close(master)
print(json.dumps({'approval_exit':p.returncode,'approval_output':out.decode(errors='replace')}),flush=True);assert p.returncode==0
ctl=P('/sys/fs/cgroup/cgroup.subtree_control');ctl.write_text('+memory +pids')
log=open(project/'.kineti/owner-acceptance/service.log','w');server=subprocess.Popen([str(binary),'task','serve','--owner-store',str(base),'--id',g['id']],cwd=project,stdout=log,stderr=log)
socket=base/'sockets'/(a.run_id+'.sock')
completed=False
try:
 for _ in range(600):
  if socket.exists() or server.poll() is not None:break
  time.sleep(.1)
 assert socket.exists(),(server.poll(),(project/'.kineti/owner-acceptance/service.log').read_text())
 args=['--socket',str(socket),'--id',g['id'],'--session',g['session']]
 alternate=cmd(['task','serve','--owner-store','/var/lib/kineti-owner-other','--id',g['id']]);assert alternate['exit']==3 and not P('/var/lib/kineti-owner-other').exists()
 duplicate=cmd(['task','serve','--owner-store',str(base),'--id',g['id']]);assert duplicate['exit']==3
 print(json.dumps({'single_store_boundary_passed':True,'alternate_store':alternate,'duplicate_service':duplicate}),flush=True)

 run=cmd(['task','run',*args]);print(json.dumps({'run':run}),flush=True)
 print(json.dumps({'status':cmd(['task','status',*args]),'original_value':(project/'value.txt').read_text(),'work_value':(base/'work'/a.run_id/'value.txt').read_text()}),flush=True)
 print(json.dumps({'reuse':cmd(['task','run',*args])}),flush=True)
 # Freshness and identity denials do not create new quotas.
 assert json.loads(run['stdout'])['result']['reason']=='checks-fresh-and-passing'
 caps=json.loads(run['stdout'])['result']['state']['host_capabilities'];assert caps['native_mcp']=='kineti_task' and caps['model_turns']==0 and caps['authenticated'] is False and len(caps['tools'])==5
 assert (project/'value.txt').read_text()=='broken\n'
 work=base/'work'/a.run_id
 assert (work/'value.txt').read_text()=='fixed\n'
 (work/'untracked.txt').write_text('changed task input\n')
 stale=cmd(['task','run',*args]);sr=json.loads(stale['stdout'])['result'];assert sr['state']['used_checks']==3 and sr['state']['used_repairs']==1
 forged=cmd(['task','run',*args[:-1],'wrong-session']);assert forged['exit']==3
 peer=subprocess.run(['su','-s','/bin/sh',a.task_user,'-c','cd '+shlex.quote(str(work))+' && '+shlex.join([str(binary),'task','run',*args])],capture_output=True,text=True);assert peer.returncode==3
 messages=[{'jsonrpc':'2.0','id':1,'method':'initialize','params':{'protocolVersion':'2024-11-05'}},{'jsonrpc':'2.0','id':2,'method':'tools/list'},{'jsonrpc':'2.0','id':3,'method':'tools/call','params':{'name':'kineti_task_status','arguments':{}}},{'jsonrpc':'2.0','id':4,'method':'tools/call','params':{'name':'kineti_evidence_record','arguments':{}}}]
 mcp=subprocess.run([str(binary),'mcp','--root',str(work),'--task-socket',str(socket),'--task-id',g['id'],'--task-session',g['session']],cwd=work,input=''.join(json.dumps(m)+'\n' for m in messages),capture_output=True,text=True)
 replies=[json.loads(l) for l in mcp.stdout.splitlines()];assert len(replies[1]['result']['tools'])==5 and not replies[2]['result']['isError'] and replies[3]['result']['isError']
 pause=cmd(['task','pause',*args]);assert pause['exit']==0
 server.wait(timeout=10)
 server=subprocess.Popen([str(binary),'task','serve','--owner-store',str(base),'--id',g['id']],cwd=project,stdout=log,stderr=log)
 for _ in range(600):
  if socket.exists():break
  time.sleep(.1)
 resumed=cmd(['task','run',*args]);rr=json.loads(resumed['stdout'])['result'];assert rr['reason']=='fresh-evidence-reused-within-valid-owner-grant' and rr['state']['used_checks']==3
 (work/'untracked.txt').write_text('stale beyond original quota\n')
 limit=cmd(['task','run',*args]);assert limit['exit']==3 and 'limit reached' in json.loads(limit['stdout'])['result']['reason']
 revoked=cmd(['task','revoke',*args]);assert revoked['exit']==0
 recovery=cmd(['task','recover',*args]);assert recovery['exit']==3
 completed=True
 print(json.dumps({'lifecycle_passed':True,'stale':stale,'forged_session':forged,'outside_group_peer_exit':peer.returncode,'pause':pause,'resumed':resumed,'limit':limit,'revoked':revoked,'recovery_after_revoke':recovery}),flush=True)
finally:
 server.terminate();server.wait(timeout=20);log.close()
 assert all(P(p).is_file() and hashlib.sha256(P(p).read_bytes()).hexdigest()==h for p,h in preserved.items()),'prior owner-store bytes changed'
 print(json.dumps({'prior_owner_store_files_preserved':len(preserved)}),flush=True)
 print(json.dumps({'server_exit':server.returncode,'server_log':(project/'.kineti/owner-acceptance/service.log').read_text()}),flush=True)


# The same product entry point also covers recovery, active detached children
# and owner-process death. These are synthetic owner-role cases, not model turns.
class Case:
 def __init__(self,suffix,rollback=False,command=None,attempts=3):
  self.id=a.run_id+suffix
  self.project=P('/var/lib/kineti-owner-acceptance-'+self.id)
  assert not self.project.exists();self.project.mkdir(mode=0o755)
  self.before={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in base.rglob('*') if p.is_file() and not p.is_symlink()}
  self.call=lambda argv:cmd(argv,cwd=self.project)
  assert self.call(['init','--goal','Disposable owner service fixture','--apply'])['exit']==0
  (self.project/'value.txt').write_text('broken\n');(self.project/'untracked.txt').write_text('preserve original\n')
  state=json.loads((self.project/'.kineti/state.json').read_text())
  self.grant=json.loads(json.dumps(g));self.grant.update(id=self.id,project_root=str(self.project),project_identity=hashlib.sha256((str(self.project)+'\0'+state['root_goal']+'\0'+state['root_goal_locked_at']).encode()).hexdigest(),automatic_rollback=rollback)
  self.grant['limits'].update(expires_at=int(time.time())+600,wall_seconds=300,command_seconds=30,verification_attempts=attempts)
  if command:self.grant['checks'][0]['argv']=command
  self.evidence=self.project/'.kineti/owner-acceptance';self.evidence.mkdir(mode=0o700)
  self.work=base/'work'/self.id;self.socket=base/'sockets'/(self.id+'.sock');self.args=['--socket',str(self.socket),'--id',self.id,'--session',self.grant['session']]
  self.log=None;self.server=None
 def __enter__(self):
  req=self.evidence/'request.json';req.write_text(json.dumps(self.grant))
  master,slave=pty.openpty();child=subprocess.Popen([str(binary),'task','approve','--owner-store',str(base),'--request',str(req)],stdin=slave,stdout=slave,stderr=slave,cwd=self.project);os.close(slave);output=b'';sent=False;deadline=time.monotonic()+15
  while child.poll() is None and time.monotonic()<deadline:
   if select.select([master],[],[],.1)[0]:
    try:output+=os.read(master,65536)
    except OSError:break
   if b'2. Deny' in output and not sent:os.write(master,b'1\n');sent=True
  if child.poll() is None:child.kill()
  child.wait();os.close(master);(self.evidence/'approval.txt').write_bytes(output);assert child.returncode==0
  self.log=(self.evidence/'service.log').open('w');self.start();return self
 def start(self,expect_fenced=False):
  self.server=subprocess.Popen([str(binary),'task','serve','--owner-store',str(base),'--id',self.id],cwd=self.project,stdout=self.log,stderr=self.log)
  end=time.monotonic()+10
  ready=False
  while time.monotonic()<end and self.server.poll() is None:
   if self.socket.exists() and self.call(['task','status',*self.args])['exit']==0:ready=True;break
   time.sleep(.05)
  if not ready and expect_fenced:
   self.server.wait(timeout=5);assert self.server.returncode==3;return False
  assert ready and self.server.poll() is None
  return True
 def request(self,method):
  r=self.call(['task',method,*self.args]);(self.evidence/(method+'-'+str(time.time_ns())+'.json')).write_text(json.dumps(r));return r
 def __exit__(self,*exception):
  if self.server and self.server.poll() is None:self.server.terminate();self.server.wait(timeout=15)
  if self.log:self.log.close()
  assert all(P(p).is_file() and hashlib.sha256(P(p).read_bytes()).hexdigest()==h for p,h in self.before.items()),'pre-existing owner records changed'
  assert (self.project/'value.txt').read_text()=='broken\n'

with Case('-recovery',rollback=True) as case:
 result=case.request('run');assert result['exit']==0
 restored=case.request('recover');assert restored['exit']==0
 assert (case.work/'value.txt').read_text()=='broken\n'
 assert (case.work/'untracked.txt').read_text()=='preserve original\n'
 print(json.dumps({'successful_known_recovery':True}),flush=True)
with Case('-conflict',rollback=True) as case:
 assert case.request('run')['exit']==0
 (case.work/'untracked.txt').write_text('concurrent user work\n')
 assert case.request('recover')['exit']==3
 assert (case.work/'untracked.txt').read_text()=='concurrent user work\n'
 assert (case.work/'value.txt').read_text()=='fixed\n'
 print(json.dumps({'concurrent_work_preserved':True}),flush=True)

probe_source=r"""#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <errno.h>
static const char *roles[]={"ordinary","setpgid","setsid","doublefork"};
static void ready(const char *role,long deadline,int error){char name[80],final[80];snprintf(name,sizeof name,"%s.ready.tmp",role);snprintf(final,sizeof final,"%s.ready",role);FILE*f=fopen(name,"w");if(!f)_exit(9);fprintf(f,"{\"pid\":%d,\"pgid\":%d,\"sid\":%d,\"uid\":%d,\"deadline\":%ld,\"error\":%d}",getpid(),getpgrp(),getsid(0),getuid(),deadline,error);fclose(f);if(rename(name,final))_exit(11);}
int main(int argc,char**argv){int seconds=argc>1?atoi(argv[1]):8;if(seconds<1||seconds>10)return 2;long deadline=time(0)+seconds;ready("leader",deadline,0);for(int i=0;i<4;i++){pid_t p=fork();if(p<0)return 3;if(!p){int error=0;if(i==1&&setpgid(0,0))error=errno;if((i==2||i==3)&&setsid()<0)error=errno;if(i==3){pid_t next=fork();if(next<0)_exit(4);if(next>0)_exit(0);}ready(roles[i],deadline,error);while(time(0)<deadline)usleep(10000);char name[80];snprintf(name,sizeof name,"%s.late",roles[i]);FILE*f=fopen(name,"w");if(!f)_exit(10);fprintf(f,"allowed");fclose(f);_exit(error?5:0);}}
while(time(0)<=deadline)usleep(10000);printf("1 pass\n0 fail\n");return 0;}
"""
probe_file=project/'.kineti/owner-acceptance/descendants.c';probe_file.write_text(probe_source)
probe_binary=project/'.kineti/owner-acceptance/descendants'
subprocess.run(['/usr/bin/cc',str(probe_file),'-o',str(probe_binary)],check=True,capture_output=True,text=True)
roles=['ordinary','setpgid','setsid','doublefork']
def seed_probe(case):
 import shutil
 shutil.copyfile(probe_binary,case.project/'descendants');(case.project/'descendants').chmod(0o755)
def ready_set(case):
 end=time.monotonic()+10
 while time.monotonic()<end:
  if all((case.work/(name+'.ready')).is_file() for name in ['leader',*roles]):break
  time.sleep(.03)
 values={name:json.loads((case.work/(name+'.ready')).read_text()) for name in ['leader',*roles]}
 assert all(value['error']==0 and value['uid']==a.task_uid for value in values.values())
 assert values['ordinary']['pgid']==values['leader']['pgid']
 for name in ['setpgid','setsid','doublefork']:assert values[name]['pgid']!=values['leader']['pgid']
 for name in ['setsid','doublefork']:assert values[name]['sid']!=values['leader']['sid']
 return values
with Case('-oracle',command=['./descendants','2'],attempts=1) as case:
 seed_probe(case);result=case.request('run');assert result['exit']==3
 assert all((case.work/(name+'.late')).read_text()=='allowed' for name in roles)
 print(json.dumps({'four_descendant_write_controls':True}),flush=True)
for suffix,crash in [('-active',False),('-crash',True)]:
 with Case(suffix,command=['./descendants','8'],attempts=1) as case:
  seed_probe(case);running=subprocess.Popen([str(binary),'task','run',*case.args],cwd=case.project,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
  values=ready_set(case);deadline=max(value['deadline'] for value in values.values())
  if crash:
   case.server.kill();case.server.wait(timeout=10)
  else:assert case.request('revoke')['exit']==0
  stdout,stderr=running.communicate(timeout=12);assert running.returncode==3
  (case.evidence/'interrupted-run.json').write_text(json.dumps({'exit':running.returncode,'stdout':stdout,'stderr':stderr,'ready':values,'owner_crash':crash},indent=2))
  while time.time()<=deadline+1:time.sleep(.05)
  assert all(not (case.work/(name+'.late')).exists() for name in roles)
  if crash:
   # Reopen only to fence the interrupted record and inspect it. No action or
   # quota is replayed, reset, renewed or silently approved.
   assert case.start(expect_fenced=True) is False
   inspection=case.call(['task','inspect','--owner-store',str(base),'--id',case.id]);assert inspection['exit']==0
   observed=json.loads(inspection['stdout'])['result'];status=observed['state']
   assert observed['read_only'] is True and observed['task_authority'] is False and observed['active_marker_present'] is False
   assert status['terminal'] is True and status['pending_execution'] is True and status['group'] is None and status['service_open'] is False
   (case.evidence/'inspection.json').write_text(json.dumps(inspection,indent=2))
   assert case.request('run')['exit']==3
  print(json.dumps({'active_descendants_stopped':True,'owner_crash':crash}),flush=True)
summary={'schema':'kineti-owner-service-offline-acceptance-v1','verified':True,'native_binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'sdk_sha256':hashlib.sha256(host.read_bytes()).hexdigest(),'run_id':a.run_id,'checks':['owner-role-approval','copied-workspace-fail-repair-retest','five-tool-mcp-catalog','forged-session-denied','outside-group-peer-denied','freshness-reuse','pause-and-resume','persistent-attempt-and-repair-limits','known-recovery','concurrent-edit-refusal','four-descendant-positive-controls','active-descendant-revocation','owner-crash-descendant-stop','interrupted-replay-refusal','read-only-owner-inspection','prior-owner-store-preserved'],'scripted_owner_console':True,'human_identity_proven':False,'provider_logins':0,'model_turns':0,'new_paid_usd_micros':0,'full_contract_accepted':False}
(project/'.kineti/owner-acceptance/result.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary),flush=True)

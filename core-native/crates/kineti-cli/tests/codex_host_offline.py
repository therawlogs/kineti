#!/usr/bin/env python3
"""Exercise the pinned real Codex unified-exec/MCP tool handlers using canned SSE.

No inference, login, key, subscription allowance, or paid service is used. The
provider override exists only in the test child; it is restricted to a loopback
HTTP listener and cannot persist user provider settings. Child commands keep
stock named filesystem/network permissions and native cgroup v2 cleanup.

Requires Linux root, enabled cgroup memory/pids controllers, a dedicated existing
unprivileged fixture account with no auth.json, the official pinned Codex runtime
and its matching helper, and a built native Kineti executable. Run fresh --case
names to preserve existing artifacts. 'no-snapshot-*' cases test the adapter fix;
'mcp-*' cases test status metadata; 'mcp-write-control-*' must remain blocked.
These mocked protocol checks are not authenticated model or installed acceptance.
"""
import os,json,time,subprocess,threading,queue,shlex,http.server,ctypes,sys,argparse,pwd
from pathlib import Path
parser=argparse.ArgumentParser(description='Linux root-only, deterministic Codex tool regression. No model service or login is used.')
parser.add_argument('--case',required=True)
parser.add_argument('--codex',required=True,type=Path)
parser.add_argument('--native',required=True,type=Path)
parser.add_argument('--task-user',required=True)
parser.add_argument('--expect',required=True,choices=['pass','blocked'])
options=parser.parse_args();assert os.geteuid()==0
assert options.case.replace('-','').isalnum() and len(options.case)<=48
actor=pwd.getpwnam(options.task_user);assert actor.pw_uid!=0
uid=actor.pw_uid;user=options.task_user
CASE=options.case
BASE=Path('/var/lib/kineti-offline-repro-'+CASE);assert not BASE.exists();BASE.mkdir(mode=0o711);W=BASE/'work';W.mkdir(mode=0o755);os.chown(W,uid,actor.pw_gid);R=BASE/'runtime';R.mkdir(mode=0o700);os.chown(R,uid,actor.pw_gid)
HOST=options.codex.resolve();NATIVE=str(options.native.resolve());assert HOST.is_file() and options.native.is_file()
assert not (Path(actor.pw_dir)/'.codex/auth.json').exists()
requests=[];events=[];responses=0
class Handler(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_POST(self):
  global responses
  body=self.rfile.read(int(self.headers.get('Content-Length',0)));assert self.headers.get('Authorization') is None
  try:data=json.loads(body)
  except Exception:data={'body_bytes':len(body),'content_encoding':self.headers.get('Content-Encoding')}
  requests.append({'path':self.path,'request':data});responses+=1;assert responses<=4
  def ev(t,**kwargs):return {'type':t,**kwargs}
  seq=[ev('response.created',response={'id':'offline-'+str(responses)})]
  if responses==1:
   if CASE.startswith('mcp'):
    tools=[x for x in data.get('tools',[]) if x.get('name')=='mcp__kineti_task'];assert len(tools)==1;assert any(x.get('name')=='kineti_task_status' for x in tools[0]['tools']);name='kineti_task_run' if 'write-control' in CASE else 'kineti_task_status';arguments={}
   else:name='exec_command';arguments={'cmd':"printf 'terminal-ok\\n'; printf 'ready\\n' > observed",'yield_time_ms':1000,'max_output_tokens':1000}
   item={'type':'function_call','call_id':'offline-exec','name':name,'arguments':json.dumps(arguments)}
   if CASE.startswith('mcp'):item['namespace']='mcp__kineti_task'
   seq.append(ev('response.output_item.done',item=item))
  else:seq.append(ev('response.output_item.done',item={'type':'message','role':'assistant','id':'offline-final','content':[{'type':'output_text','text':'Offline protocol fixture finished; no model inference.'}]}))
  seq.append(ev('response.completed',response={'id':'offline-'+str(responses),'usage':{'input_tokens':0,'output_tokens':0,'total_tokens':0}}))
  payload=''.join('event: '+e['type']+'\ndata: '+json.dumps(e)+'\n\n' for e in seq).encode();self.send_response(200);self.send_header('Content-Type','text/event-stream');self.send_header('Content-Length',str(len(payload)));self.end_headers();self.wfile.write(payload)
server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler);threading.Thread(target=server.serve_forever,daemon=True).start()
cg=Path('/sys/fs/cgroup')/('kineti-task-offline-'+str(os.getpid()));Path('/sys/fs/cgroup/cgroup.subtree_control').write_text('+memory +pids');cg.mkdir();(cg/'memory.max').write_text('1073741824');(cg/'pids.max').write_text('128');(cg/'memory.swap.max').write_text('0')
profile={'filesystem':{':minimal':'read',str(BASE):'deny',str(W):'write',str(HOST):'read',str(HOST.with_name('codex-code-mode-host')):'read'},'network':{'enabled':False}}
def toml(v):
 if isinstance(v,dict):return '{'+','.join(json.dumps(k)+'='+toml(x) for k,x in v.items())+'}'
 return json.dumps(v)
config={'cli_auth_credentials_store':'ephemeral','features.apps':False,'features.plugins':False,'features.code_mode_host':True,'analytics.enabled':False,'project_doc_max_bytes':0,'log_dir':str(R/'logs'),'sqlite_home':str(R/'db'),'default_permissions':'offline','permissions.offline':profile,'model_provider':'offline_fixture','model':'gpt-5.4','model_providers.offline_fixture':{'name':'Deterministic local protocol fixture, no inference','base_url':'http://127.0.0.1:'+str(server.server_port)+'/v1','wire_api':'responses','requires_openai_auth':False,'supports_websockets':False},'mcp_servers':{},'web_search':'disabled'}
if CASE.startswith('no-snapshot'):config['features.shell_snapshot']=False
if CASE.startswith('nonlogin'):config['allow_login_shell']=False

if CASE.startswith('mcp'):
 import socket,struct
 ipc=BASE/'task.sock';listener=socket.socket(socket.AF_UNIX);listener.bind(str(ipc));ipc.chmod(0o666);listener.listen(1)
 def ipc_serve():
  c,_=listener.accept()
  with c:
   pid,uid,gid=struct.unpack('3i',c.getsockopt(socket.SOL_SOCKET,socket.SO_PEERCRED,12));data=b''
   while b'\n' not in data:data+=c.recv(65536)
   req=json.loads(data);assert uid==actor.pw_uid and req['method']=='status' and req['grant_id']=='offline-task' and req['session']=='offline-session' and req['workspace']==str(W)
   assert ('0::/'+cg.name) in Path('/proc/'+str(pid)+'/cgroup').read_text().splitlines()
   (BASE/'ipc-called').write_text('status')
   c.sendall((json.dumps({'schema':'kineti-task-decision-v1','decision':'allow','action':'read-status','source':'offline fixture root authority, actual kernel peer binding','new_paid_spending':0,'release_authority':False})+'\n').encode())
 threading.Thread(target=ipc_serve,daemon=True).start()
 native=NATIVE
 config['mcp_servers']={'kineti_task':{'command':native,'args':['mcp','--root',str(W),'--task-socket',str(ipc),'--task-id','offline-task','--task-session','offline-session'],'required':True}}
 config['features.shell_snapshot']=False
args=[str(HOST),'app-server','--stdio']
for k,v in config.items():args+=['-c',k+'='+toml(v)]
def pre():
 (cg/'cgroup.procs').write_text(str(os.getpid()));assert ctypes.CDLL(None).prctl(38,1,0,0,0)==0
p=subprocess.Popen(['su','-s','/bin/sh',user,'-c','exec '+shlex.join(args)],cwd=W,env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','TZ':'UTC'},stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,preexec_fn=pre)
q=queue.Queue();seq=0;pending={};stderr=[];approvals=[]
def reader():
 for line in p.stdout:
  try:q.put(json.loads(line))
  except Exception:break
 q.put({'eof':True})
def errreader():
 for line in p.stderr:stderr.append(line)
threading.Thread(target=reader,daemon=True).start();threading.Thread(target=errreader,daemon=True).start()
def pump():
 try:m=q.get(timeout=.2)
 except queue.Empty:return
 if m.get('eof'):raise RuntimeError('app-server-eof')
 if 'id' in m and 'method' not in m:pending[m['id']]=m
 else:
  events.append(m)
  if 'id' in m:
   approvals.append(m);reply={'jsonrpc':'2.0','id':m['id'],'error':{'code':-32001,'message':'Offline fixture does not approve expanded scope'}};p.stdin.write(json.dumps(reply)+'\n');p.stdin.flush()
def call(method,params):
 global seq
 seq+=1;i=seq;p.stdin.write(json.dumps({'jsonrpc':'2.0','id':i,'method':method,'params':params})+'\n');p.stdin.flush();end=time.monotonic()+30
 while i not in pending:
  assert time.monotonic()<end,method;pump()
 v=pending.pop(i);assert 'error' not in v,v;return v['result']
result={'case':CASE,'real_model_calls':0,'logins':0,'temporary_provider':'loopback-only process override; no persisted provider settings'}
try:
 call('initialize',{'clientInfo':{'name':'kineti-offline-root-cause','version':'1'},'capabilities':{'experimentalApi':True}});p.stdin.write('{"method":"initialized"}\n');p.stdin.flush();a=call('account/read',{'refreshToken':False});assert a['account'] is None
 t=call('thread/start',{'cwd':str(W),'ephemeral':True,'model':'gpt-5.4','modelProvider':'offline_fixture','permissions':'offline','approvalPolicy':'on-request','approvalsReviewer':'user','allowProviderModelFallback':False,'config':{'project_doc_max_bytes':0}});assert t['modelProvider']=='offline_fixture';tid=t['thread']['id'];call('turn/start',{'threadId':tid,'input':[{'type':'text','text':'Execute only the deterministic local tool fixture.'}],'permissions':'offline','approvalPolicy':'on-request','approvalsReviewer':'user','cwd':str(W)})
 end=time.monotonic()+60
 while not any(e.get('method')=='turn/completed' for e in events):assert time.monotonic()<end,'offline turn timeout';pump()
 result.update({'observed_file':(W/'observed').read_text() if (W/'observed').exists() else None,'events':events,'http_requests':requests,'stderr':''.join(stderr),'scope_requests':approvals,'mcp_readonly_metadata':None,'passed':(BASE/'ipc-called').exists() if CASE.startswith('mcp') else (W/'observed').exists()})
finally:
 kill=subprocess.run([NATIVE,'host','revoke','--cgroup',str(cg),'--task-uid',str(uid),'--wait-ms','1000'],capture_output=True,text=True,timeout=5);assert kill.returncode==0;p.wait(timeout=5);cg.rmdir();server.shutdown()
 print(json.dumps(result),flush=True)
assert result.get('passed') == (options.expect=='pass'), 'offline regression expectation failed'
if options.expect=='pass':assert not approvals, 'passing operation requested expanded scope'
if 'write-control' in CASE:assert approvals, 'mutating MCP operation lost its consent gate'

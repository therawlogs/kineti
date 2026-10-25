"""Generate public compatibility goldens with exact main-line TypeScript readers.
Requires Git history and Bun for development only; native installation needs neither.
All disposable files stay inside the project. No real accounting is opened.
"""
from pathlib import Path
import hashlib, json, subprocess
root = Path(__file__).resolve().parents[1]
commit = '307331ad37451b174f31b40bc0da04b544685ed0'
work = root / '.kineti/reconciliation-060/original-main-fixtures'
work.mkdir(parents=True, exist_ok=True)
sources = {}
for name in ['spending-state.ts', 'lib.ts', 'ledger-integrity.ts', 'kineti-evidence.ts']:
    raw = subprocess.check_output(['git', 'show', f'{commit}:bin/{name}'], cwd=root)
    sources[name] = hashlib.sha256(raw).hexdigest()
    if name != 'kineti-evidence.ts':
        (work / name).write_bytes(raw)
    else:
        text = raw.decode()
        start = text.index('export function unsignedRecord(')
        end = text.index('\n/**', start)
        (work / 'original-signer.ts').write_text('import crypto from "node:crypto";\ntype EvidenceRecord = any;\n' + text[start:end])
script = r'''
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {readSpending,spendingStop} from './spending-state.ts';
import {unsignedRecord,verifyReceiptSignature} from './original-signer.ts';
const key=crypto.createPrivateKey({key:Buffer.from('302e020100300506032b657004220420'+'42'.repeat(32),'hex'),format:'der',type:'pkcs8'});
const public_key=crypto.createPublicKey(key).export({format:'pem',type:'spki'}).toString();
const receipts=[];
for(const [name,extra] of [['session-parent',{parent_run_id:'cccccccc-cccc-cccc-cccc-cccccccccccc'}],['controller-operational',{result_kind:'check',verification:{status:'unsupported'},harness:{status:'unknown',observed_partial_cost_usd:0.01}}]]) {
 const record:any={receipt_version:3,at:'2026-10-25T00:00:00.000Z',label:name,session_id:'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',run_id:'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',manifest:{command:['runner','','two words','--x=y','தமிழ் 😀','quote"\n']},...extra,issuer_key:public_key};
 const unsigned=unsignedRecord(record);record.signature=crypto.sign(null,Buffer.from(unsigned),key).toString('base64');
 if(!verifyReceiptSignature(record,public_key))throw Error('original signature rejected');
 receipts.push({name,record,unsigned});
}
const run='aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',session='bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb';
const reservation={kind:'reservation',run_id:run,session_id:session,stage:'build',microcents:1000000};
const settlement={kind:'settlement',run_id:run,session_id:session,stage:'build',microcents:250000,usd:0.25,source:'public fixture reported cost',provider_invoice_verified:false};
const cancelled={kind:'reservation-cancelled',run_id:run,session_id:session,stage:'build'};
const pending={ [run]:{session_id:session,stage:'build',microcents:1000000,status:'reserved'}};
const zero={tripped:false,reason:null,total_usd:0,total_microcents:0,by_stage:{},by_stage_microcents:{},entries:0,routing_reservations:{}};
const settled={...zero,total_usd:0.25,total_microcents:250000,by_stage:{build:0.25},by_stage_microcents:{build:250000},entries:1};
const cases:any[]=[['verified-empty-seed',{total_microcents:0,tripped:false},[]],['settled',settled,[reservation,settlement]],['settlement-rounding',{...settled,total_usd:0.00005,total_microcents:50,by_stage:{build:0.00005},by_stage_microcents:{build:50}},[reservation,{...settlement,usd:0.0000499,microcents:50}]],['cancelled',zero,[reservation,cancelled]],['pending-reserved',{...zero,routing_reservations:pending},[reservation]],['pending-unknown-partial',{...zero,routing_reservations:{[run]:{...pending[run],status:'unknown',observed_partial_cost_usd:0.01}}},[reservation]],['snapshot-only',{...zero,routing_reservations:pending},[]],['log-only',zero,[reservation]],['orphan-settlement',settled,[settlement]],['duplicate-reservation',settled,[reservation,reservation,settlement]],['foreign-session',settled,[reservation,{...settlement,session_id:'cccccccc-cccc-cccc-cccc-cccccccccccc'}]],['foreign-stage',settled,[reservation,{...settlement,stage:'ship'}]],['null-map',{...zero,routing_reservations:null},[]],['array-map',{...zero,routing_reservations:[]},[]],['unknown-kind',zero,[{kind:'invented'}]],['wrong-total',{...settled,total_microcents:250001},[reservation,settlement]],['wrong-pending-amount',{...zero,routing_reservations:{[run]:{...pending[run],microcents:999999}}},[reservation]],['duplicate-settlement',settled,[reservation,settlement,settlement]],['unsafe-reservation',{...zero,routing_reservations:pending},[{...reservation,microcents:Number.MAX_SAFE_INTEGER+1}]]];
const histories=[];
for(const [name,state,log] of cases){
 const dir=path.join(import.meta.dir,'projects',name);fs.mkdirSync(path.join(dir,'.kineti'),{recursive:true});
 fs.writeFileSync(path.join(dir,'.kineti/spend.json'),JSON.stringify(state));fs.writeFileSync(path.join(dir,'.kineti/spend.log.jsonl'),log.map(v=>JSON.stringify(v)).join('\n')+(log.length?'\n':''));
 let original_read_accepts=false,original_check_passes=false,total_microcents=null;
 try{const s=readSpending(dir);original_read_accepts=true;total_microcents=s.total_microcents;original_check_passes=spendingStop(s,{globalUsd:75,safetyFactor:0.95,perStage:{},perStageDefaultUsd:75})===null;}catch{}
 histories.push({name,state,log,original_read_accepts,original_check_passes,total_microcents});
}
console.log(JSON.stringify({public_key,receipts,histories}));
'''
(work / 'generate.ts').write_text(script)
data = json.loads(subprocess.check_output(['bun', str(work / 'generate.ts')], cwd=root))
data['provenance'] = {'commit': commit, 'source_sha256': sources, 'method': 'Exact original TypeScript reader/stop and unsignedRecord/verifyReceiptSignature; public test-only key; disposable fixture projects only.'}
# Original read tolerates null via ?? but its stop policy rejects it. Preserve fail-closed behavior.
for case in data['histories']:
    if case['name'] in ['settled', 'cancelled']:
        assert case['original_check_passes'], case['name']
out = root / 'core-native/crates/kineti-cli/tests/fixtures/main-legacy-golden.json'
out.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
print(f'Generated {len(data["receipts"])} signed receipts and {len(data["histories"])} original accounting decisions')

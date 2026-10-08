"""Collect a ga-eval-team workflow result (task output JSON) into evals/ and the ledger.
usage: collect_wf.py <workflow-output-file>"""
import json, os, sys, subprocess
from collections import defaultdict
GA=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,GA); import ga
d=json.load(open(sys.argv[1])); res=d.get('result',d)
by=defaultdict(list); bad=[]
for r in res:
    if r is None: continue
    if r.get('findings') is None: bad.append(f"{r['cand']}/{r['lens']}"); continue
    for f in r['findings']: f['lens']=r['lens']; f['model']=r.get('model','default')
    by[r['cand']]+=r['findings']
if bad: print('no result:', bad)
led=ga.ledger()
for cand,arr in by.items():
    fp=f'{GA}/evals/{cand}.findings.json'
    old=json.load(open(fp)) if os.path.exists(fp) else []
    arr=old+arr; json.dump(arr,open(fp,'w'),indent=1)
    out=subprocess.run(['python3',f'{GA}/attribute.py',f'{GA}/out/{cand}',fp],capture_output=True,text=True).stdout.strip()
    per=json.loads(out) if out else {}
    asm=json.load(open(f'{GA}/out/{cand}/assembly.json'))
    nf=sum(1 for f in arr if f.get('severity')=='needs-fix')
    lenses=sorted({f['lens'] for f in arr})
    entry={'name':cand,'features':asm['genome']['features'],'needs_fix':nf,'minor':len(arr)-nf,'per_feature':per,'dangling':len(asm['dangling']),'lenses':lenses}
    led=[e for e in led if e['name']!=cand]+[entry]
    print(cand,'needs-fix',nf,'minor',len(arr)-nf,per)
json.dump(led,open(ga.LEDGER,'w'),indent=1)

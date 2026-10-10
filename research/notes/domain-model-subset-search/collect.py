"""Collect evaluator hand-backs listed in evals/<batch>.txt, attribute findings, update the ledger.
usage: collect.py <batch>"""
import json, re, os, sys, subprocess
from collections import defaultdict
GA=os.path.dirname(os.path.abspath(__file__))
T=os.environ.get('GA_TASKS', '')  # folder of the evaluator agents' .output files, ending in /
sys.path.insert(0,GA); import ga
def handback(aid):
    p=T+aid+'.output'
    if not os.path.exists(p): return None
    last=None
    for line in open(p):
        try: o=json.loads(line)
        except: continue
        c=(o.get('message') or {}).get('content')
        if isinstance(c,list):
            for x in c:
                if isinstance(x,dict) and x.get('type')=='tool_use' and x.get('name')=='SubagentHandback':
                    last=(x.get('input') or {}).get('message')
    return last
batch=sys.argv[1]
rows=[l.split() for l in open(f'{GA}/evals/{batch}.txt') if l.strip()]
by=defaultdict(list); missing=[]
for cand,lens,aid in rows:
    msg=handback(aid)
    if msg is None: missing.append(f'{cand}/{lens}'); continue
    open(f'{GA}/evals/{cand}.{lens}.md','w').write(msg)
    m=re.findall(r'```json\s*(.*?)```', msg, re.S)
    try: arr=json.loads(m[-1]) if m else []
    except Exception as e: print('bad json',cand,lens,e); arr=[]
    for f in arr: f['lens']=lens
    by[cand]+=arr
if missing: print('missing:', missing)
led=ga.ledger()
for cand,arr in by.items():
    if any(f'{cand}/' in x for x in missing): continue
    fp=f'{GA}/evals/{cand}.findings.json'; json.dump(arr,open(fp,'w'),indent=1)
    out=subprocess.run(['python3',f'{GA}/attribute.py',f'{GA}/out/{cand}',fp],capture_output=True,text=True).stdout.strip()
    per=json.loads(out) if out else {}
    asm=json.load(open(f'{GA}/out/{cand}/assembly.json'))
    nf=sum(1 for f in arr if f.get('severity')=='needs-fix')
    entry={'name':cand,'features':asm['genome']['features'],'needs_fix':nf,'minor':len(arr)-nf,'per_feature':per,'dangling':len(asm['dangling'])}
    led=[e for e in led if e['name']!=cand]+[entry]
    print(cand, 'needs-fix', nf, 'minor', len(arr)-nf, per)
json.dump(led,open(ga.LEDGER,'w'),indent=1)

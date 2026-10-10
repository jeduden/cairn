import json, sys, os, re
T=os.environ.get('REVIEW_TASKS', '')  # folder of the reviewer agents' .output files, ending in /
OUT=os.path.dirname(os.path.abspath(__file__))+'/'
ids=dict(a.split('=') for a in sys.argv[1:])
for name, aid in ids.items():
    p=T+aid+'.output'
    if not os.path.exists(p): print(name,'missing'); continue
    last=None; final=False
    for line in open(p):
        try: o=json.loads(line)
        except: continue
        m=o.get('message') or {}
        if o.get('type')=='assistant' or m.get('role')=='assistant':
            c=m.get('content')
            if isinstance(c,list):
                for x in c:
                    if isinstance(x,dict) and x.get('type')=='tool_use' and x.get('name')=='SubagentHandback':
                        last=(x.get('input') or {}).get('message'); final=True
            elif isinstance(c,str) and c.strip(): last=c
    if not last: print(name,'running'); continue
    open(OUT+name+'.md','w').write(last)
    m=re.findall(r'```json\s*(.*?)```', last, re.S)
    n='?'
    if m:
        try: n=len(json.loads(m[-1]))
        except Exception as e: n='bad json: '+str(e)[:60]
    print(name, len(last), 'findings:', n)

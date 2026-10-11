"""Inventory of the domain model's list items: one entry per list item (any depth)."""
import re, json, os, sys
ROOT=os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../../docs/domain-model')
def items(path):
    lines=open(path).read().split('\n')
    out=[]; cur=None
    for i,l in enumerate(lines):
        m=re.match(r'^(\s*)- (.*)$', l)
        if m:
            if cur: out.append(cur)
            cur={'indent':len(m.group(1)),'start':i,'lines':[l]}
        elif cur and l.strip() and (len(l)-len(l.lstrip()))>cur['indent']:
            cur['lines'].append(l)
        else:
            if cur: out.append(cur); cur=None
    if cur: out.append(cur)
    for it in out:
        t=' '.join(x.strip() for x in it['lines'])
        it['text']=t
        bolds=re.findall(r'\*\*([^*]+)\*\*', t)
        it['head']=bolds[0].rstrip(':').strip() if bolds else t[2:40]
        it['defines']=[b.rstrip(':').strip() for b in bolds]
    return out
if __name__=='__main__':
    inv=[]
    for f in sorted(os.listdir(ROOT)):
        if not f.endswith('.md'): continue
        for k,it in enumerate(items(os.path.join(ROOT,f))):
            inv.append({'file':f[:-3],'n':k,'indent':it['indent'],'line':it['start']+1,'head':it['head'],'defines':it['defines'],'chars':len(it['text'])})
    json.dump(inv,open(os.path.join(os.path.dirname(__file__),'inventory.json'),'w'),indent=1)
    from collections import Counter
    print(len(inv),'items', Counter(i['file'] for i in inv))
    print('defined terms', sum(len(i['defines']) for i in inv))

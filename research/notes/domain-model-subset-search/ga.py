"""Genome bookkeeping for the subset search: validity, repair, mutation,
crossover, fitness and the ledger of evaluated candidates."""
import json, os, random, sys
GA=os.path.dirname(os.path.abspath(__file__))
F=json.load(open(f'{GA}/features.json'))
FEATS={f['id']:f for f in F['features']}
LEDGER=f'{GA}/ledger.json'

def closure_add(gen):
    g=set(gen); changed=True
    while changed:
        changed=False
        for f in list(g):
            for r in FEATS[f]['requires']:
                if r not in g: g.add(r); changed=True
            for grp in FEATS[f].get('requires_any',[]):
                if not (set(grp)&g):
                    pick=min(grp,key=lambda x:(FEATS[x]['risk'],-FEATS[x]['value'])); g.add(pick); changed=True
    return g

def closure_drop(gen):
    g=set(gen); changed=True
    while changed:
        changed=False
        for f in list(g):
            ok=all(r in g for r in FEATS[f]['requires']) and all(set(grp)&g for grp in FEATS[f].get('requires_any',[]))
            if not ok: g.discard(f); changed=True
    return g

def value(g): return sum(FEATS[f]['value'] for f in g)

def ledger():
    return json.load(open(LEDGER)) if os.path.exists(LEDGER) else []

def fitness(entry, penalty=4.0, dpen=0.5):
    return value(entry['features']) - penalty*entry['needs_fix'] - dpen*entry.get('dangling',0)

def blame():
    """Per-feature needs-fix attributions, normalised by how often the feature was evaluated."""
    b={}; n={}
    for e in ledger():
        for f in e['features']: n[f]=n.get(f,0)+1
        for f,c in e.get('per_feature',{}).items(): b[f]=b.get(f,0)+c
    return {f: b.get(f,0)/max(1,n.get(f,0)) for f in FEATS}, b

def mutate(g, k=2, rng=random):
    rate,_=blame(); g=set(g)
    for _ in range(k):
        if g and rng.random()<0.5:
            # drop: prefer blamed features
            cand=sorted(g,key=lambda f:-rate.get(f,0)+rng.random()*0.5)
            g.discard(cand[0]); g=closure_drop(g)
        else:
            out=[f for f in FEATS if f not in g]
            if not out: continue
            cand=sorted(out,key=lambda f:rate.get(f,0)-FEATS[f]['value']*0.2+rng.random()*0.5)
            g.add(cand[0]); g=closure_add(g)
    return g

def crossover(a,b,rng=random):
    a,b=set(a),set(b)
    child=(a&b)|{f for f in a^b if rng.random()<0.5}
    return closure_drop(closure_add(child))

if __name__=='__main__':
    cmd=sys.argv[1]
    if cmd=='blame':
        r,b=blame(); print(json.dumps(sorted(((round(v,2),f) for f,v in r.items() if v),reverse=True)))
    elif cmd=='rank':
        for e in sorted(ledger(),key=lambda e:-fitness(e)):
            print(f"{fitness(e):7.1f} val={value(e['features']):3d} nf={e['needs_fix']:3d} d={e.get('dangling',0):3d} n={len(e['features']):2d} {e['name']}")

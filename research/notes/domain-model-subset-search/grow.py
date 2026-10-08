"""Growth step: from a base genome, write one genome per single-feature
addition (closed under requires), skipping additions already in the base.
Usage: grow.py <base.json> <prefix>  -> genomes/<prefix>-<feature>.json"""
import json, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ga import FEATS, closure_add
base = json.load(open(sys.argv[1])); prefix = sys.argv[2]
have = set(base['features']); seen = set(); out = []
for f in sorted(FEATS):
    if f in have: continue
    g = closure_add(have | {f}); key = tuple(sorted(g - have))
    if key in seen: continue
    seen.add(key)
    name = f'{prefix}-{f}'
    json.dump({'name': name, 'features': sorted(g), 'added': list(key)},
              open(f'genomes/{name}.json', 'w'), indent=1)
    out.append((name, list(key)))
for n, k in out: print(n, k)

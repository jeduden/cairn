"""Gather the probe reviews of one GA generation into the ledger.
usage: collect_probes.py <tag>
Reads evals/<tag>-*.probe.json ({"findings": [...]}, written by each probe
reviewer), writes evals/<tag>.results.json and runs collect_wf.py on it."""
import glob, json, os, subprocess, sys
GA = os.path.dirname(os.path.abspath(__file__))
tag = sys.argv[1]
res = []
for p in sorted(glob.glob(f'{GA}/evals/{tag}-*.probe.json')):
    cand = os.path.basename(p)[:-len('.probe.json')]
    try:
        findings = json.load(open(p))['findings']
    except Exception as e:  # a malformed file is reported, never silently dropped
        print(f'{cand}: unreadable probe file: {e}')
        findings = None
    res.append({'cand': cand, 'lens': 'all', 'model': 'sonnet', 'findings': findings})
out = f'{GA}/evals/{tag}.results.json'
json.dump(res, open(out, 'w'), indent=1)
subprocess.run(['python3', f'{GA}/collect_wf.py', out], check=True)

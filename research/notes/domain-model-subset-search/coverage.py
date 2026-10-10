import json, os, sys, collections
HERE = os.path.dirname(os.path.abspath(__file__))
items = [i["id"] for i in json.load(open(os.path.join(HERE, "items.json")))]
d = json.load(open(os.path.join(HERE, "features.json")))
feats = [d["core"]] + d["features"]
ids = [f["id"] for f in feats]
out = []
assign = collections.defaultdict(list)
for f in feats:
    for e in f["entries"]:
        assign[e].append(f["id"])
unassigned = [i for i in items if i not in assign]
dups = {e: v for e, v in assign.items() if len(v) > 1}
unknown = [e for e in assign if e not in items]
badreq = [(f["id"], r) for f in feats for r in f["requires"] + sum(f.get("requires_any", []), []) + f.get("soft", []) if r not in ids]
# cycle check
g = {f["id"]: f["requires"] for f in feats}
state = {}
cyc = []
def dfs(n, path):
    if state.get(n) == 1: cyc.append(path + [n]); return
    if state.get(n) == 2: return
    state[n] = 1
    for m in g.get(n, []): dfs(m, path + [n])
    state[n] = 2
for n in g: dfs(n, [])
empty = [f["id"] for f in d["features"] if not f["entries"]]
for f in feats:
    for k in ["id","name","description","entries","requires","value","risk","notes"]:
        assert k in f, (f["id"], k)
    assert 1 <= f["value"] <= 5 and 1 <= f["risk"] <= 5
core_missing_just = [e for e in d["core"]["entries"] if not d["core"]["justification"].get(e)]
out.append(f"model list items: {len(items)} (concept files {sum(not i.startswith('hub-') for i in items)}, hub Relations {sum(i.startswith('hub-') for i in items)})")
out.append(f"features: core + {len(d['features'])}; core entries {len(d['core']['entries'])}")
out.append(f"unassigned items: {unassigned or 'none'}")
out.append(f"items in more than one feature: {dups or 'none'}")
out.append(f"entries naming no model item: {unknown or 'none'}")
out.append(f"dangling feature ids in requires/requires_any/soft: {badreq or 'none'}")
out.append(f"requires cycles: {cyc or 'none'}")
out.append(f"core entries without a justification: {core_missing_just or 'none'}")
out.append(f"features with no entries of their own: {empty}")
out.append("not assigned by design (not concepts, not asked for): the hub's verb list (owns, is a member of, has a seat in, holds) and the six 'Names follow the model' rules; both hold for any subset and belong with core.")
txt = "\n".join(out)
open(os.path.join(HERE, "coverage.txt"), "w").write(txt + "\n")
print(txt)

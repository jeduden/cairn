import re, json, sys, os
D = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../../docs/domain-model')
stems = ["acts-and-roles","components-and-surfaces","git-and-forge","harness-facts","pins-and-context","places","principals-and-agents","record","seats-and-keys","trust-and-flow"]
items = []
for s in stems:
    txt = open(os.path.join(D, s + ".md")).read().split("\n")
    in_fm = False
    for i, line in enumerate(txt):
        if i == 0 and line == "---": in_fm = True; continue
        if in_fm:
            if line == "---": in_fm = False
            continue
        m = re.match(r'^( *)- (.*)$', line)
        if not m: continue
        indent = len(m.group(1))
        b = re.search(r'\*\*(.+?)\*\*', m.group(2))
        term = b.group(1) if b else None
        items.append({"id": f"{s}: {term}", "indent": indent, "line": i+1})
# hub relations
txt = open(os.path.join(D, "index.md")).read().split("\n")
sec = None
for i, line in enumerate(txt):
    if line.startswith("## "): sec = line[3:]
    m = re.match(r'^- (.*)$', line)
    if m and sec == "Relations":
        words = " ".join(m.group(1).split()[:5])
        items.append({"id": f"hub-relation: {words}", "indent": 0, "line": i+1})
for it in items: print(it["indent"], it["line"], it["id"])
print(len(items), file=sys.stderr)
ids = [it["id"] for it in items]
import collections
print([k for k,v in collections.Counter(ids).items() if v>1], file=sys.stderr)
json.dump(items, open(os.path.join(os.path.dirname(__file__), "items.json"), "w"), indent=1)

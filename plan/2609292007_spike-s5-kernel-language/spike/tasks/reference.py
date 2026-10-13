"""Reference answers for the spike S5 tasks, computed in plain Python.

    python3 reference.py record.json > answers.json
"""

import json
import re
import sys
from collections import Counter, defaultdict

E = json.load(open(sys.argv[1]))
E.sort(key=lambda e: (e["writer"], e["seq"]))


def args(e):
    return json.loads(e["args"]) if e["args"] else {}


calls = [e for e in E if e["kind"] == "tool_call"]
results = [e for e in E if e["kind"] == "tool_result"]
result_by = {r["call_id"]: r for r in results}
user = [e for e in E if e["kind"] == "message" and e["provenance"] == "user"]
runs = sorted({e["run"] for e in E if e["run"] is not None})
writers = sorted({e["writer"] for e in E})
by_writer = defaultdict(list)
for e in E:
    by_writer[e["writer"]].append(e)
by_run = defaultdict(list)
for e in E:
    if e["run"] is not None:
        by_run[e["run"]].append(e)


def rank(counter, *keys):
    return sorted(counter.items(), key=lambda kv: (-kv[1], kv[0]))


A = {}
A["t01"] = len(E)
A["t02"] = sum(1 for c in calls if c["tool"] == "Bash")
A["t03"] = sum(1 for r in results if r["status"] == "error")
A["t04"] = len({args(c)["file_path"] for c in calls if c["tool"] in ("Edit", "Write")})
A["t05"] = sum(1 for u in user if "test" in u["text"].lower())
A["t06"] = [list(x) for x in rank(Counter(c["tool"] for c in calls))]
toks = Counter()
for e in E:
    if e["kind"] == "message" and e["provenance"] == "assistant":
        toks[e["run"]] += e["tokens"]
A["t07"] = [list(x) for x in rank(toks)[:3]]
err, tot = Counter(), Counter()
for r in results:
    tot[r["tool"]] += 1
    err[r["tool"]] += r["status"] == "error"
A["t08"] = sorted(([t, err[t], tot[t], err[t] / tot[t]] for t in tot), key=lambda x: (-x[3], x[0]))
comp = Counter(e["run"] for e in E if e["kind"] == "compaction")
A["t09"] = [[r, comp[r]] for r in runs]
A["t10"] = [list(x) for x in rank(Counter(args(c)["file_path"] for c in calls if c["tool"] == "Read"))[:5]]
A["t11"] = [[w, len(by_writer[w]), max(e["seq"] for e in by_writer[w])] for w in writers]
dur = defaultdict(list)
for c in calls:
    r = result_by.get(c["call_id"])
    if r:
        dur[c["tool"]].append(r["ts"] - c["ts"])
A["t12"] = [[t, sum(v) / len(v)] for t, v in sorted(dur.items())]
A["t13"] = sum(1 for c in calls if c["call_id"] not in result_by)
pairs = [(result_by[c["call_id"]]["ts"] - c["ts"], c["call_id"]) for c in calls if c["call_id"] in result_by]
A["t14"] = sorted(pairs, key=lambda p: (-p[0], p[1]))[0][1]
runs_by, fails_by = Counter(), Counter()
for c in calls:
    if c["tool"] == "Bash":
        cmd = args(c)["command"]
        runs_by[cmd] += 1
        r = result_by.get(c["call_id"])
        fails_by[cmd] += bool(r and r["status"] == "error")
A["t15"] = sorted(([k, runs_by[k], fails_by[k]] for k in runs_by if fails_by[k] > 0), key=lambda x: (-x[2], x[0]))
n = 0
for w in writers:
    read = set()
    for e in by_writer[w]:
        if e["kind"] != "tool_call":
            continue
        if e["tool"] == "Read":
            read.add(args(e)["file_path"])
        elif e["tool"] == "Edit" and args(e)["file_path"] not in read:
            n += 1
A["t16"] = n
hours = Counter((u["ts"] % 86400) // 3600 for u in user)
A["t17"] = [[h, hours[h]] for h in sorted(hours)]
span = {r: max(e["ts"] for e in v) - min(e["ts"] for e in v) for r, v in by_run.items()}
A["t18"] = sorted(([r, s] for r, s in span.items()), key=lambda x: (-x[1], x[0]))
A["t19"] = len({u["ts"] // 86400 for u in user})
best = None
for w in writers:
    ev = by_writer[w]
    for a, b in zip(ev, ev[1:]):
        cand = (-(b["ts"] - a["ts"]), w, b["seq"])
        if best is None or cand < best:
            best = cand
A["t20"] = [[best[1], best[2], -best[0]]]
A["t21"] = sum(args(e)["pre_tokens"] for e in E if e["kind"] == "compaction" and args(e)["trigger"] == "auto")
codes = Counter()
for r in results:
    if r["tool"] == "Bash" and r["status"] == "error":
        codes[int(re.search(r"exit code (\d+)", r["text"]).group(1))] += 1
A["t22"] = [[c, codes[c]] for c in sorted(codes)]
q = Counter(args(e)["query"] for e in E if e["kind"] == "recall")
A["t23"] = sorted(k for k, v in q.items() if v > 1)
ext = Counter(args(c)["file_path"].rsplit(".", 1)[1] for c in calls if c["tool"] == "Edit")
A["t24"] = [list(x) for x in rank(ext)]
longest = 0
for w in writers:
    cur = 0
    for e in by_writer[w]:
        if e["kind"] == "tool_result":
            cur = cur + 1 if e["status"] == "error" else 0
            longest = max(longest, cur)
A["t25"] = longest
s = 0
for r, ev in by_run.items():
    ev = sorted(ev, key=lambda e: e["seq"])
    s += 1 + sum(1 for a, b in zip(ev, ev[1:]) if b["ts"] - a["ts"] > 1800)
A["t26"] = s
active = set()
for e in E:
    if e["kind"] == "pin":
        a = args(e)
        if a["action"] == "add":
            active.add(a["pin_id"])
        else:
            active.discard(a["pin_id"])
A["t27"] = sorted(active)
turns = Counter()
for w in writers:
    cur_run, in_turn, bad = None, False, False
    for e in by_writer[w]:
        if e["kind"] == "message" and e["provenance"] == "user":
            if in_turn and bad:
                turns[cur_run] += 1
            in_turn, bad, cur_run = True, False, e["run"]
        elif in_turn and e["kind"] == "tool_result" and e["status"] == "error":
            bad = True
    if in_turn and bad:
        turns[cur_run] += 1
A["t28"] = [[r, turns[r]] for r in sorted(turns) if turns[r] > 0]
retries = 0
for w in writers:
    prev = None
    for e in by_writer[w]:
        if e["kind"] != "tool_call":
            continue
        if prev and prev["tool"] == e["tool"] and prev["args"] == e["args"]:
            r = result_by.get(prev["call_id"])
            if r and r["status"] == "error":
                retries += 1
        prev = e
A["t29"] = retries
out = []
for r in runs:
    total = 0
    for e in sorted(by_run[r], key=lambda e: e["seq"]):
        total += e["tokens"]
        if total >= 50000:
            out.append([r, e["seq"]])
            break
A["t30"] = out

json.dump(A, sys.stdout, indent=1, sort_keys=True)
sys.stdout.write("\n")

"""Score the spike S5 solutions against the reference answers.

    python3 score.py <spike-dir>

Runs every solution under solutions/<lang>/r<rep>/<task>.<ext> through
the runner, compares its answer with tasks/answers.json, and writes
results.json. A first attempt that failed with an error may have a
repair, <task>.repair.<ext>, written after the solver saw the error;
it is scored as the second attempt.
"""

import json
import math
import os
import subprocess
import sys
from collections import defaultdict

EXT = {"lua": "lua", "sql": "sql"}


def run(runner, lang, record, path):
    out = subprocess.run([runner, lang, record, path], capture_output=True,
                         text=True, timeout=60)
    try:
        return json.loads(out.stdout)
    except json.JSONDecodeError:
        return {"ok": False, "error": "runner: " + (out.stderr or out.stdout)[-400:]}


def shape(lang, kind, value):
    """The answer in the reference's shape, or a reason it has none."""
    if lang == "lua":
        return value, None
    rows = value
    if kind == "scalar":
        if len(rows) != 1 or len(rows[0]) != 1:
            return None, "expected one row with one column, got %d rows" % len(rows)
        return rows[0][0], None
    if kind == "list":
        if any(len(r) != 1 for r in rows):
            return None, "expected one column"
        return [r[0] for r in rows], None
    return rows, None


def same(a, b):
    if isinstance(b, bool) or isinstance(a, bool):
        return a == b
    if isinstance(b, (int, float)) and isinstance(a, (int, float)):
        return abs(a - b) <= 1e-6 * max(1.0, abs(b))
    if isinstance(b, list) and isinstance(a, list):
        return len(a) == len(b) and all(same(x, y) for x, y in zip(a, b))
    return a == b


def verdict(lang, task, answer, out):
    if not out.get("ok"):
        return "error", out.get("error", "")
    value, why = shape(lang, task["shape"], out["value"])
    if why:
        return "wrong", why
    return ("pass" if same(value, answer) else "wrong"), None


def wilson(k, n, z=1.959964):
    if n == 0:
        return (0.0, 0.0)
    p = k / n
    d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (max(0.0, c - h), min(1.0, c + h))


def mcnemar(b, c):
    """Exact two-sided McNemar p-value on b and c discordant pairs."""
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    tail = sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n
    return min(1.0, 2 * tail)


def main():
    spike = sys.argv[1]
    runner = os.path.join(spike, "runner/target/release/s5-runner")
    record = os.path.join(spike, "fixture/record.json")
    tasks = json.load(open(os.path.join(spike, "tasks/tasks.json")))
    answers = json.load(open(os.path.join(spike, "tasks/answers.json")))
    rows = []
    for lang in ("lua", "sql"):
        for rep in (1, 2, 3):
            d = os.path.join(spike, "solutions", lang, "r%d" % rep)
            for t in tasks:
                first = os.path.join(d, "%s.%s" % (t["id"], EXT[lang]))
                repair = os.path.join(d, "%s.repair.%s" % (t["id"], EXT[lang]))
                if not os.path.exists(first):
                    v1, why1 = "missing", "no solution file"
                else:
                    v1, why1 = verdict(lang, t, answers[t["id"]], run(runner, lang, record, first))
                v2, why2 = v1, why1
                if v1 == "error" and os.path.exists(repair):
                    v2, why2 = verdict(lang, t, answers[t["id"]], run(runner, lang, record, repair))
                rows.append({"lang": lang, "rep": rep, "task": t["id"],
                             "category": t["category"], "first": v1,
                             "first_detail": why1, "repaired": v2,
                             "repaired_detail": why2})
    json.dump(rows, open(os.path.join(spike, "results.json"), "w"), indent=1)
    summarize(rows, tasks)


def summarize(rows, tasks):
    for attempt in ("first", "repaired"):
        print("== %s attempt ==" % attempt)
        for lang in ("lua", "sql"):
            rs = [r for r in rows if r["lang"] == lang]
            k = sum(r[attempt] == "pass" for r in rs)
            lo, hi = wilson(k, len(rs))
            kinds = defaultdict(int)
            for r in rs:
                kinds[r[attempt]] += 1
            print("%-4s %3d/%d = %5.1f%%  95%% CI %5.1f%%-%5.1f%%  %s" % (
                lang, k, len(rs), 100 * k / len(rs), 100 * lo, 100 * hi, dict(kinds)))
        cats = sorted({t["category"] for t in tasks})
        for c in cats:
            line = "  %-9s" % c
            for lang in ("lua", "sql"):
                rs = [r for r in rows if r["lang"] == lang and r["category"] == c]
                k = sum(r[attempt] == "pass" for r in rs)
                lo, hi = wilson(k, len(rs))
                line += "  %s %2d/%2d (%3.0f%%, CI %3.0f-%3.0f)" % (
                    lang, k, len(rs), 100 * k / len(rs), 100 * lo, 100 * hi)
            print(line)
        key = {(r["lang"], r["rep"], r["task"]): r[attempt] == "pass" for r in rows}
        b = sum(1 for (l, rep, t), ok in key.items() if l == "lua" and ok and not key[("sql", rep, t)])
        c = sum(1 for (l, rep, t), ok in key.items() if l == "sql" and ok and not key[("lua", rep, t)])
        print("  paired: lua-only passes %d, sql-only passes %d, McNemar p = %.3g" % (b, c, mcnemar(b, c)))
        print("  per task (lua/sql passes of 3):")
        for t in tasks:
            l = sum(key[("lua", rep, t["id"])] for rep in (1, 2, 3))
            s = sum(key[("sql", rep, t["id"])] for rep in (1, 2, 3))
            print("    %s %-9s lua %d  sql %d" % (t["id"], t["category"], l, s))


if __name__ == "__main__":
    main()

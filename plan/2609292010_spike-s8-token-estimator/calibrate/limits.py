"""Measure recorded Claude Code transcripts and hook inputs against the
input limits the SRS marks as initial targets: SEC-05's hook stdin and
JSON depth, REC-09's payload threshold, REC-11's indexing cap and LMK-03's
structural field length. Prints aggregates only, never content.

Usage: python3 -I limits.py DIR...
  Reads every *.jsonl under each DIR as transcript lines and every *.json
  under a hooks/ directory as a hook input.
"""

import json
import os
import sys


def depth(v):
    if isinstance(v, dict):
        return 1 + max((depth(x) for x in v.values()), default=0)
    if isinstance(v, list):
        return 1 + max((depth(x) for x in v), default=0)
    return 0


def pct(xs, p):
    xs = sorted(xs)
    if not xs:
        return 0
    return xs[min(len(xs) - 1, max(0, -(-len(xs) * p // 100) - 1))]


def content_texts(line):
    """The text of each content block a line carries: what an event holds."""
    msg = line.get("message")
    if not isinstance(msg, dict):
        return
    content = msg.get("content")
    if isinstance(content, str):
        yield content
        return
    for block in content or []:
        if not isinstance(block, dict):
            continue
        if block.get("type") == "text":
            yield block.get("text", "")
        elif block.get("type") == "tool_use":
            yield json.dumps(block.get("input"), ensure_ascii=False)
        elif block.get("type") == "tool_result":
            c = block.get("content")
            if isinstance(c, str):
                yield c
            else:
                yield "".join(x.get("text", "") for x in c or [] if isinstance(x, dict))


def fields(line):
    """Structural fields untrusted input can influence (LMK-03)."""
    msg = line.get("message")
    if not isinstance(msg, dict) or not isinstance(msg.get("content"), list):
        return
    for block in msg["content"]:
        if isinstance(block, dict) and block.get("type") == "tool_use":
            name = block.get("name", "")
            yield "tool", name
            if name.startswith("mcp__"):
                yield "mcp server", name.split("__")[1]
            inp = block.get("input") or {}
            for k in ("file_path", "path", "notebook_path"):
                if isinstance(inp.get(k), str):
                    yield "path", inp[k]


def main():
    hooks, lines, texts, depths = [], [], [], []
    sizes = {"tool": [], "mcp server": [], "path": []}
    for root in sys.argv[1:]:
        for d, _, names in os.walk(root):
            for n in names:
                p = os.path.join(d, n)
                if n.endswith(".json") and "/hooks/" in p:
                    raw = open(p, "rb").read()
                    hooks.append((len(raw), depth(json.loads(raw))))
                elif n.endswith(".jsonl"):
                    for raw in open(p, "rb"):
                        try:
                            v = json.loads(raw)
                        except ValueError:
                            continue
                        lines.append(len(raw))
                        depths.append(depth(v))
                        texts.extend(len(t.encode("utf-8")) for t in content_texts(v))
                        for k, f in fields(v):
                            sizes[k].append(len(f))
    hb = [h[0] for h in hooks]
    if hooks:
        print(f"hook inputs: {len(hooks)}, bytes max {max(hb)}, p99 {pct(hb, 99)}; JSON depth max {max(h[1] for h in hooks)}")
    print(f"transcript lines: {len(lines)}, bytes p50 {pct(lines, 50)} p95 {pct(lines, 95)} p99 {pct(lines, 99)} max {max(lines)}")
    print(f"  JSON depth p99 {pct(depths, 99)} max {max(depths)}")
    big = [t for t in texts if t > 8192]
    print(f"content blocks: {len(texts)}, bytes p50 {pct(texts, 50)} p95 {pct(texts, 95)} p99 {pct(texts, 99)} max {max(texts)}")
    print(f"  above 8 KiB: {len(big)} ({100 * len(big) / len(texts):.1f}%), holding {100 * sum(big) / sum(texts):.1f}% of content bytes")
    print(f"  above 1 MiB: {sum(t > 1 << 20 for t in texts)}")
    for k, xs in sizes.items():
        if xs:
            print(f"{k}: {len(xs)}, length p99 {pct(xs, 99)} max {max(xs)}, above 80: {sum(x > 80 for x in xs)}")


if __name__ == "__main__":
    main()

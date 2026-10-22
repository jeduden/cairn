"""Draw spike S8's sample of record content.

Writes sample.jsonl: one object per sample with its id, stratum, source and
text. measure.py adds Claude's model-token counts. The draw is seeded, and
every source is the repository at a pinned commit or text generated from
the seed, so a rerun at the same commit draws the same windows except for
the command outputs that depend on the checkout's file times.

Usage: python3 -I collect.py REPO_ROOT COMMIT OUT.jsonl
"""

import base64
import hashlib
import json
import math
import os
import random
import re
import subprocess
import sys
import uuid

SEED = 2609292010
MIN_BYTES = 8
MAX_BYTES = 16384

COUNTS = {
    "prose": 110,
    "code": 110,
    "transcript": 110,
    "tool-output": 100,
    "high-entropy": 80,
}
MULTILINGUAL_JOINS = 24

PROSE = ["*.md"]
CODE = ["*.rs", "*.toml", "*.yml", "*.yaml", "*.sh", "*.py", "*.go", "*.feature"]
TRANSCRIPT = ["testdata/**/*.jsonl", "testdata/**/*.json"]

MENTION = re.compile(r"(?:^|\s)@(\S+)")
KEYWORDS = re.compile(r"ultrathink|ultracode|think harder|megathink", re.I)


def git(repo, *args):
    return subprocess.run(
        ["git", "-C", repo, *args], check=True, capture_output=True
    ).stdout.decode("utf-8", "replace")


def files(repo, commit, patterns):
    out = git(repo, "ls-tree", "-r", "--name-only", commit)
    names = out.splitlines()
    picked = []
    for name in names:
        for p in patterns:
            if match(name, p):
                picked.append(name)
                break
    return sorted(picked)


def match(name, pattern):
    if pattern.startswith("testdata/**/"):
        return name.startswith("testdata/") and name.endswith(pattern[len("testdata/**/*"):])
    return name.endswith(pattern[1:]) and not name.startswith("testdata/")


def show(repo, commit, name):
    return git(repo, "show", f"{commit}:{name}")


def target_len(rng):
    return int(math.exp(rng.uniform(math.log(MIN_BYTES), math.log(MAX_BYTES))))


def cut(text, start, nbytes):
    raw = text.encode("utf-8")[start:start + nbytes]
    return raw.decode("utf-8", "ignore")


def window(rng, text, nbytes):
    """A window of about nbytes starting at a line start."""
    raw = text.encode("utf-8")
    if len(raw) <= nbytes:
        return text
    start = rng.randrange(len(raw) - nbytes + 1)
    if start:
        nl = raw.find(b"\n", start - 1)
        start = nl + 1 if 0 <= nl < len(raw) - 1 else 0
    return cut(text, start, nbytes)


def mention_resolves(text, home, work):
    for m in MENTION.finditer(text):
        path = m.group(1).strip("\"'`,.;:)]}")
        if path.startswith("~"):
            path = home + path[1:]
        elif not path.startswith("/"):
            path = os.path.join(work, path)
        if path and os.path.exists(path):
            return True
    return False


def usable(text, home, work):
    if not text.strip():
        return False
    if KEYWORDS.search(text):
        return False
    return not mention_resolves(text, home, work)


def tool_outputs(repo, commit):
    """Real command output over the pinned commit."""
    log = git(repo, "log", "--stat", "-n", "300", commit)
    fuller = git(repo, "log", "--format=fuller", "-n", "300", commit)
    shas = git(repo, "rev-list", "-n", "40", commit).split()
    diffs = "".join(git(repo, "show", "--format=medium", s) for s in shas)
    tree = git(repo, "ls-tree", "-r", "-l", commit)
    grep = git(repo, "grep", "-n", "-e", "fn ", "-e", "MUST", commit, "--", "crates", "tooling", "docs/srs")
    blame = git(repo, "blame", "-l", commit, "--", "Cargo.toml") + git(
        repo, "blame", commit, "--", "crates/cairn/src/main.rs"
    )
    listing = subprocess.run(
        ["ls", "-laR", "--time-style=long-iso", "crates", "tooling"],
        cwd=repo, check=True, capture_output=True,
    ).stdout.decode("utf-8", "replace")
    return [log, fuller, diffs, tree, grep, blame, listing]


def high_entropy(rng, nbytes):
    kind = rng.choice(["base64", "hex", "uuids", "sha256", "xxd", "jwt"])
    out = []
    size = 0
    while size < nbytes:
        if kind == "base64":
            line = base64.b64encode(rng.randbytes(57)).decode()
        elif kind == "hex":
            line = rng.randbytes(32).hex()
        elif kind == "uuids":
            line = str(uuid.UUID(bytes=rng.randbytes(16), version=4))
        elif kind == "sha256":
            name = f"src/{rng.choice(['store', 'hook', 'mcp', 'pin'])}/mod{rng.randrange(100)}.rs"
            line = f"{hashlib.sha256(rng.randbytes(16)).hexdigest()}  {name}"
        elif kind == "xxd":
            off = size // 64 * 16
            data = rng.randbytes(16)
            pairs = " ".join(data[i:i + 2].hex() for i in range(0, 16, 2))
            text = "".join(chr(b) if 32 <= b < 127 else "." for b in data)
            line = f"{off:08x}: {pairs}  {text}"
        else:
            parts = [base64.urlsafe_b64encode(rng.randbytes(n)).decode().rstrip("=") for n in (27, 96, 32)]
            line = ".".join(parts)
        out.append(line)
        size += len(line) + 1
    return cut("\n".join(out), 0, nbytes), kind


def authored(path):
    blocks = open(path, encoding="utf-8").read().split("\n%%\n")[1:]
    for block in blocks:
        stratum, _, text = block.partition("\n")
        yield stratum.strip(), text.rstrip("\n")


def main():
    repo, commit, out = sys.argv[1], sys.argv[2], sys.argv[3]
    here = os.path.dirname(os.path.abspath(__file__))
    home = os.environ.get("S8_HOME", "/nonexistent-home")
    work = os.environ.get("S8_WORK", "/nonexistent-work")
    rng = random.Random(SEED)
    samples = []
    rejected = 0

    def add(stratum, source, text):
        nonlocal rejected
        if not usable(text, home, work):
            rejected += 1
            return False
        samples.append({"id": f"s{len(samples):04d}", "stratum": stratum, "source": source, "text": text})
        return True

    pools = {
        "prose": [(n, show(repo, commit, n)) for n in files(repo, commit, PROSE)],
        "code": [(n, show(repo, commit, n)) for n in files(repo, commit, CODE)],
        "transcript": [(n, show(repo, commit, n)) for n in files(repo, commit, TRANSCRIPT)],
        "tool-output": list(zip(
            ["git log --stat", "git log --format=fuller", "git show", "git ls-tree -r -l",
             "git grep -n", "git blame", "ls -laR"],
            tool_outputs(repo, commit),
        )),
    }
    for stratum, pool in pools.items():
        pool = [(n, t) for n, t in pool if t.strip()]
        taken = 0
        while taken < COUNTS[stratum]:
            name, text = rng.choice(pool)
            if add(stratum, name, window(rng, text, target_len(rng))):
                taken += 1

    taken = 0
    while taken < COUNTS["high-entropy"]:
        text, kind = high_entropy(rng, target_len(rng))
        if add("high-entropy", f"generated:{kind}", text):
            taken += 1

    multilingual = []
    for stratum, text in authored(os.path.join(here, "authored.txt")):
        add(stratum, "authored.txt", text)
        if stratum == "multilingual":
            multilingual.append(text)
    for _ in range(MULTILINGUAL_JOINS):
        parts = rng.sample(multilingual, rng.randint(2, 8))
        add("multilingual", "authored.txt:joined", "\n\n".join(parts))

    with open(out, "w", encoding="utf-8") as f:
        for s in samples:
            f.write(json.dumps(s, ensure_ascii=False) + "\n")
    print(f"{len(samples)} samples, {rejected} rejected draws", file=sys.stderr)


if __name__ == "__main__":
    main()

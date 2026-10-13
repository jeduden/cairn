"""Fit the S8 estimator's weights on the committed sample.

The estimator (bench/src/estimate.rs) sorts each character into one of
eleven classes and counts the switches between lowercase, uppercase and
digit inside a run of letters and digits. Its estimate is

    ceil(sum(w_f * n_f) / 1024) + BASE

over those twelve features f. The fit is a quantile regression on the ratio
estimate / count, so each sample weighs alike whatever its length: it
minimizes the mean relative overestimate while leaving at most a share
1 - TAU of the samples under their count. Each character class has a floor
(FLOOR), so a class the sample holds little of never comes out free. The
fitted weights are multiplied by MARGIN and rounded up to whole 1/1024ths.

Five-fold cross validation repeats the fit, margin included, without one
fifth of the samples and reports the share of that fifth it covers.

The count fitted is the reference model's (REFERENCE), which every sample
has. The tokenizer is shared: every other model measured gave the same
count, except two Fable 5.1 counts the report explains.

Usage: python3 fit.py SAMPLE.jsonl
Needs numpy and scipy.
"""

import json
import sys
import zlib

import numpy as np
from scipy.optimize import linprog

REFERENCE = "claude-haiku-5-5"
TAU = 0.995
MARGIN = 1.1
FEATURES = [
    "letter", "digit", "space", "break", "punct",
    "latin", "two", "three", "symbol", "cjk", "four",
    "switch",
]
# Model tokens per character at least: a digit group holds at most three
# digits, a non-ASCII character at least half a token, a four-byte one a
# whole token.
FLOOR = [0.15, 0.34, 0.02, 0.1, 0.25, 0.5, 0.5, 0.5, 0.5, 0.5, 1.0, 0.0]


def klass(c):
    """The character class, as bench/src/estimate.rs's `class`."""
    o = ord(c)
    if o < 0x80:
        if c.isalpha():
            return 0
        if c.isdigit():
            return 1
        if c == " ":
            return 2
        if o < 0x20 or o == 0x7F:
            return 3
        return 4
    if o < 0x300:
        return 5
    if o < 0x800:
        return 6
    if 0x2000 <= o < 0x2C00:
        return 8
    if 0x2E80 <= o < 0xA000 or 0xAC00 <= o < 0xD7B0 or 0xF900 <= o < 0xFB00 or 0xFF00 <= o < 0xFFF0:
        return 9
    if o < 0x10000:
        return 7
    return 10


def kind(c):
    if "a" <= c <= "z":
        return 1
    if "A" <= c <= "Z":
        return 2
    if "0" <= c <= "9":
        return 3
    return 0


def features(text):
    x = np.zeros(len(FEATURES))
    prev = 0
    for c in text:
        x[klass(c)] += 1
        k = kind(c)
        if k and prev and k != prev:
            x[11] += 1
        prev = k
    return x


def fit(X, y):
    """Weights w >= FLOOR and base b >= 0 minimizing the TAU-quantile loss
    of (X w + b) / y - 1."""
    n, k = X.shape
    A = np.hstack([X / y[:, None], 1 / y[:, None]])
    c = np.concatenate([np.zeros(k + 1), np.full(n, 1 - TAU), np.full(n, TAU)])
    A_eq = np.hstack([A, -np.eye(n), np.eye(n)])
    bounds = [(f, None) for f in FLOOR] + [(0, None)] * (1 + 2 * n)
    res = linprog(c, A_eq=A_eq, b_eq=np.ones(n), bounds=bounds, method="highs")
    if not res.success:
        sys.exit(res.message)
    w, b = res.x[:k], res.x[k]
    return [int(np.ceil(v * MARGIN * 1024 - 1e-9)) for v in w], int(np.ceil(b - 1e-9))


def estimate(X, w, b):
    return np.ceil(X @ np.array(w) / 1024) + b


def main():
    samples = [json.loads(l) for l in open(sys.argv[1], encoding="utf-8")]
    X = np.array([features(s["text"]) for s in samples])
    y = np.array([s["tokens"][REFERENCE] for s in samples], dtype=float)
    strata = np.array([s["stratum"] for s in samples])
    folds = np.array([zlib.crc32(s["id"].encode()) % 5 for s in samples])

    held = np.zeros(len(samples), dtype=bool)
    for f in range(5):
        tr, te = folds != f, folds == f
        w, b = fit(X[tr], y[tr])
        held[te] = estimate(X[te], w, b) >= y[te]
    w, b = fit(X, y)
    est = estimate(X, w, b)
    print(f"{len(samples)} samples, tau {TAU}, margin {MARGIN}")
    print("WEIGHTS:", w)
    print("BASE:", b)
    print(f"covered on the fit: {np.mean(est >= y):.4f}; held out, five folds: {np.mean(held):.4f}")
    print(f"estimate / count over all: {est.sum() / y.sum():.2f}")
    for s in sorted(set(strata)):
        m = strata == s
        print(f"  {s:13} n={m.sum():4} fit {np.mean(est[m] >= y[m]):.3f} held {np.mean(held[m]):.3f}"
              f" estimate/count {est[m].sum() / y[m].sum():.2f}")


if __name__ == "__main__":
    main()

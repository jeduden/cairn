"""One surrogate-assisted GA round: fit per-feature needs-fix rates from the
ledger, evolve genomes against that surrogate, and write the most promising,
mutually distinct genomes for real review.

usage: ga_round.py <tag> <k> [--since v4] [--only v0] [--seed N] [--gens 300] [--pop 60]
  <tag> names the text version and generation, such as v6-ga1; writes
  genomes/<tag>-<i>.json for i in 1..k and prints the surrogate. Genomes
  already reviewed on the same text version are not picked again.

Surrogate: rate(f) = (needs-fix attributed to f + a) / (reviews including f + b)
over ledger entries whose name starts with a version at or after --since
(older text no longer exists), or only those of one version with --only. Probe reviews (one combined lens) count at
PROBE_WEIGHT of a strong one, since they find fewer issues. Each pick is a
Thompson sample: draw every feature's rate from its Gamma posterior, evolve
against that draw (fitness = value - PENALTY * predicted needs-fix), and take
the best genome not yet chosen. Rarely reviewed features draw widely, so
they get explored; well-measured ones settle."""
import json, os, random, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ga import FEATS, GA, closure_add, closure_drop, ledger, value

PENALTY = 10.0       # a needs-fix outweighs any feature's value: the goal is none
PROBE_WEIGHT = 0.5   # a probe review weighs half a strong one
A, B = 0.1, 1.0      # Gamma prior on a feature's needs-fix per review
MIN_DIST = 3         # chosen genomes differ in at least this many features


def version(name):
    """Text version a candidate was assembled from: v6-ga1-2 -> 6, g3-core -> 3."""
    head = name.split('-')[0].lstrip('vg')
    digits = ''
    for ch in head:
        if not ch.isdigit():
            break
        digits += ch
    return int(digits) if digits else -1


def fit(since, only=None):
    hits, seen = {}, {}
    for e in ledger():
        if e.get('stale'):
            continue
        v = version(e['name'])
        if (only is not None and v != only) or (only is None and v < since):
            continue
        w = PROBE_WEIGHT if e.get('lenses') == ['all'] else 1.0
        for f in e['features']:
            seen[f] = seen.get(f, 0.0) + w
        for f, c in e.get('per_feature', {}).items():
            if f in FEATS:
                hits[f] = hits.get(f, 0.0) + w * c
    return hits, seen


def draw(hits, seen, rng):
    """One Thompson sample of every feature's needs-fix rate."""
    return {f: rng.gammavariate(hits.get(f, 0.0) + A, 1.0 / (seen.get(f, 0.0) + B))
            for f in FEATS}


def score(g, rate):
    return value(g) - PENALTY * sum(rate[f] for f in g)


def mutate(g, rng):
    g = set(g)
    f = rng.choice(sorted(FEATS))
    if f in g:
        g.discard(f)
        return closure_drop(g)
    return closure_drop(closure_add(g | {f}))


def crossover(a, b, rng):
    child = (a & b) | {f for f in a ^ b if rng.random() < 0.5}
    return closure_drop(closure_add(child))


def evolve(rate, rng, gens, pop_n):
    seeds = [frozenset(e['features']) - {'core'} for e in ledger()]
    pop = {frozenset(closure_drop(closure_add(s))) for s in seeds}
    while len(pop) < pop_n:
        pop.add(frozenset(closure_drop(closure_add(
            {f for f in FEATS if rng.random() < 0.5}))))
    pop = list(pop)
    for _ in range(gens):
        pop.sort(key=lambda g: -score(g, rate))
        elite = pop[:pop_n // 4]
        kids = []
        while len(kids) < pop_n - len(elite):
            a, b = rng.sample(elite, 2) if len(elite) > 1 else (elite[0], elite[0])
            kid = crossover(set(a), set(b), rng)
            if rng.random() < 0.7:
                kid = mutate(kid, rng)
            kids.append(frozenset(kid))
        pop = elite + kids
    uniq = sorted(set(pop), key=lambda g: -score(g, rate))
    return uniq


def main():
    args = sys.argv[1:]
    tag, k = args[0], int(args[1])
    opt = {args[i]: args[i + 1] for i in range(2, len(args) - 1, 2)}
    since = version(opt.get('--since', 'v4'))
    rng = random.Random(int(opt.get('--seed', '1')))
    gens, pop_n = int(opt.get('--gens', '300')), int(opt.get('--pop', '60'))
    only = version(opt['--only']) if '--only' in opt else None
    hits, seen = fit(since, only)
    mean = {f: (hits.get(f, 0.0) + A) / (seen.get(f, 0.0) + B) for f in FEATS}
    ver = version(tag)
    done = {frozenset(e['features']) - {'core'} for e in ledger()
            if version(e['name']) == ver}
    chosen = []
    for _ in range(8 * k):
        if len(chosen) == k:
            break
        for g in evolve(draw(hits, seen, rng), rng, gens, pop_n):
            if g not in done and all(len(g ^ c) >= MIN_DIST for c in chosen):
                chosen.append(g)
                break
    for i, g in enumerate(chosen, 1):
        name = f'{tag}-{i}'
        json.dump({'name': name, 'features': sorted(g)},
                  open(f'{GA}/genomes/{name}.json', 'w'), indent=1)
        print(f'{name}: n={len(g)} value={value(g)} '
              f'expected-nf={sum(mean[f] for f in g):.1f} '
              f'out={sorted(set(FEATS) - g)}')
    print('posterior mean needs-fix per review (reviews):')
    for f in sorted(FEATS, key=lambda f: -mean[f]):
        print(f'  {f:22s} {mean[f]:.2f} ({seen.get(f, 0):.1f})')


if __name__ == '__main__':
    main()

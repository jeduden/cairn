//! The synthetic corpus: a fixed vocabulary of pseudo-words, code
//! identifiers, paths and numbers, drawn with a Zipf distribution so
//! term frequencies look like natural text. Everything is seeded.
//!
//! The generator ports Go's math/rand/v2 (PCG-DXSM, Lemire's bounded
//! draw, and the rejection-inversion Zipf) draw for draw, so a seed
//! gives the same corpus the Go harness in ../bench-go built: the two
//! reports measure the same store.

use std::collections::HashSet;

pub const VOCAB_SIZE: usize = 60000;

const SYLLABLES: &str = "ka lo mi ne ru ta shi po an el or un is ex de re co pro tra ver
	gen mod str int fmt err ctx buf sql log net cfg env tmp run set get put add del
	map key val row col idx seq hash pin rec tok sum";

/// Go's rand.PCG with rand.Rand's draws on top.
#[derive(Clone)]
pub struct Rng {
    hi: u64,
    lo: u64,
}

impl Rng {
    pub fn new(seed1: u64, seed2: u64) -> Self {
        Rng { hi: seed1, lo: seed2 }
    }

    /// The generator's seeding in gen.go: `pcg(seed)`.
    pub fn pcg(seed: u64) -> Self {
        Rng::new(seed, seed ^ 0x9e3779b97f4a7c15)
    }

    fn next(&mut self) -> (u64, u64) {
        const MUL_HI: u64 = 2549297995355413924;
        const MUL_LO: u64 = 4865540595714422341;
        const INC_HI: u64 = 6364136223846793005;
        const INC_LO: u64 = 1442695040888963407;
        let m = (self.lo as u128) * (MUL_LO as u128);
        let (mut hi, lo) = ((m >> 64) as u64, m as u64);
        hi = hi
            .wrapping_add(self.hi.wrapping_mul(MUL_LO))
            .wrapping_add(self.lo.wrapping_mul(MUL_HI));
        let (lo, c) = lo.overflowing_add(INC_LO);
        let hi = hi.wrapping_add(INC_HI).wrapping_add(c as u64);
        self.lo = lo;
        self.hi = hi;
        (hi, lo)
    }

    pub fn u64(&mut self) -> u64 {
        let (mut hi, lo) = self.next();
        const CHEAP_MUL: u64 = 0xda942042e4dd58b5;
        hi ^= hi >> 32;
        hi = hi.wrapping_mul(CHEAP_MUL);
        hi ^= hi >> 48;
        hi.wrapping_mul(lo | 1)
    }

    /// Go's Rand.uint64n on a 64-bit machine.
    pub fn below(&mut self, n: u64) -> u64 {
        if n & (n - 1) == 0 {
            return self.u64() & (n - 1);
        }
        let mul = |x: u64| {
            let m = (x as u128) * (n as u128);
            ((m >> 64) as u64, m as u64)
        };
        let (mut hi, mut lo) = mul(self.u64());
        if lo < n {
            let thresh = n.wrapping_neg() % n;
            while lo < thresh {
                (hi, lo) = mul(self.u64());
            }
        }
        hi
    }

    pub fn int_n(&mut self, n: usize) -> usize {
        self.below(n as u64) as usize
    }

    pub fn f64(&mut self) -> f64 {
        (self.u64() << 11 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Go's rand.Zipf: k in [0, imax] with P(k) proportional to (v+k)^-s.
pub struct Zipf {
    imax: f64,
    v: f64,
    q: f64,
    s: f64,
    one_minus_q: f64,
    one_minus_q_inv: f64,
    hxm: f64,
    hx0_minus_hxm: f64,
}

impl Zipf {
    pub fn new(s: f64, v: f64, imax: u64) -> Self {
        let mut z = Zipf {
            imax: imax as f64,
            v,
            q: s,
            s: 0.0,
            one_minus_q: 1.0 - s,
            one_minus_q_inv: 1.0 / (1.0 - s),
            hxm: 0.0,
            hx0_minus_hxm: 0.0,
        };
        z.hxm = z.h(z.imax + 0.5);
        z.hx0_minus_hxm = z.h(0.5) - (z.v.ln() * -z.q).exp() - z.hxm;
        z.s = 1.0 - z.hinv(z.h(1.5) - (-z.q * (z.v + 1.0).ln()).exp());
        z
    }

    fn h(&self, x: f64) -> f64 {
        (self.one_minus_q * (self.v + x).ln()).exp() * self.one_minus_q_inv
    }

    fn hinv(&self, x: f64) -> f64 {
        (self.one_minus_q_inv * (self.one_minus_q * x).ln()).exp() - self.v
    }

    pub fn draw(&self, r: &mut Rng) -> u64 {
        loop {
            let u = r.f64();
            let ur = self.hxm + u * self.hx0_minus_hxm;
            let x = self.hinv(ur);
            let k = (x + 0.5).floor();
            if k - x <= self.s {
                return k as u64;
            }
            if ur >= self.h(k + 0.5) - (-(k + self.v).ln() * self.q).exp() {
                return k as u64;
            }
        }
    }
}

/// vocab builds VOCAB_SIZE distinct terms. Rank 0 is the most frequent.
pub fn vocab(r: &mut Rng) -> Vec<String> {
    let syl: Vec<&str> = SYLLABLES.split_whitespace().collect();
    let mut seen = HashSet::with_capacity(VOCAB_SIZE);
    let mut out = Vec::with_capacity(VOCAB_SIZE);
    while out.len() < VOCAB_SIZE {
        let k = r.int_n(10);
        let w = if k < 6 {
            let n = 1 + r.int_n(4);
            let mut b = String::new();
            for _ in 0..n {
                b.push_str(syl[r.int_n(syl.len())]);
            }
            b
        } else if k < 8 {
            // Go evaluates the operands left to right.
            let a = syl[r.int_n(syl.len())];
            let b = syl[r.int_n(syl.len())][..1].to_uppercase();
            let c = syl[r.int_n(syl.len())];
            let d = syl[r.int_n(syl.len())];
            format!("{a}{b}{c}{d}")
        } else if k < 9 {
            let a = syl[r.int_n(syl.len())];
            let b = syl[r.int_n(syl.len())];
            format!("{a}_{b}{}", r.int_n(100))
        } else {
            r.int_n(1_000_000).to_string()
        };
        if seen.insert(w.to_lowercase()) {
            out.push(w);
        }
    }
    out
}

#[derive(Clone)]
pub struct Event {
    pub session: String,
    pub source: i64,
    pub line: i64,
    pub kind: &'static str,
    pub trust: i64,
    pub tool: Option<&'static str>,
    pub body: String,
    pub payload: bool,
    pub ts: i64,
}

const KINDS: [&str; 5] = ["user", "assistant", "tool_use", "tool_result", "system"];
const TOOLS: [&str; 6] = ["Bash", "Read", "Edit", "Grep", "Write", "Glob"];

pub struct Corpus {
    r: Rng,
    zipf: Zipf,
    pub words: Vec<String>,
    per_ses: i64,
    n: i64,
}

impl Corpus {
    /// The store's own corpus: vocabulary and events both drawn from seed.
    pub fn new(seed: u64) -> Self {
        Corpus::from_seeds(seed, seed)
    }

    /// Vocabulary from vocab_seed, events from stream_seed; equal seeds
    /// give `new`'s stream exactly.
    pub fn from_seeds(vocab_seed: u64, stream_seed: u64) -> Self {
        let mut r = Rng::pcg(vocab_seed);
        let words = vocab(&mut r);
        if stream_seed != vocab_seed {
            r = Rng::pcg(stream_seed);
        }
        Corpus {
            r,
            zipf: Zipf::new(1.07, 2.0, (VOCAB_SIZE - 1) as u64),
            words,
            per_ses: 5000,
            n: 0,
        }
    }

    /// An event's inline text length: most are short, some medium, a
    /// tail are 2 KiB payload previews (§8.1).
    fn body_len(&mut self) -> (usize, bool) {
        let k = self.r.int_n(100);
        if k < 60 {
            (40 + self.r.int_n(160), false)
        } else if k < 90 {
            (200 + self.r.int_n(800), false)
        } else {
            (2048, true)
        }
    }

    pub fn next(&mut self) -> Event {
        let i = self.n;
        self.n += 1;
        let ses = i / self.per_ses;
        let (n, payload) = self.body_len();
        let mut b = String::with_capacity(n + 16);
        while b.len() < n {
            if !b.is_empty() {
                if self.r.int_n(12) == 0 {
                    b.push_str(". ");
                } else {
                    b.push(' ');
                }
            }
            b.push_str(&self.words[self.zipf.draw(&mut self.r) as usize]);
        }
        let kind = KINDS[self.r.int_n(KINDS.len())];
        let tool = if kind == "tool_use" || kind == "tool_result" {
            Some(TOOLS[self.r.int_n(TOOLS.len())])
        } else {
            None
        };
        Event {
            session: format!("ses-{ses}"),
            source: ses,
            line: i % self.per_ses,
            kind,
            trust: self.r.int_n(3) as i64,
            tool,
            body: b,
            payload,
            ts: 1_780_000_000_000 + i * 37,
        }
    }
}

/// Query term bands by Zipf rank.
pub struct Band {
    pub name: &'static str,
    pub lo: usize,
    pub hi: usize,
}

pub const BANDS: [Band; 3] = [
    Band { name: "common", lo: 20, hi: 300 },
    Band { name: "mid", lo: 300, hi: 5000 },
    Band { name: "rare", lo: 5000, hi: VOCAB_SIZE },
];

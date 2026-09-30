package main

import (
	"math/rand/v2"
	"strconv"
	"strings"
)

// The synthetic corpus: a fixed vocabulary of pseudo-words, code
// identifiers, paths and numbers, drawn with a Zipf distribution so
// term frequencies look like natural text. Everything is seeded, so
// both drivers index byte-identical stores.

const vocabSize = 60000

var syllables = strings.Fields(`ka lo mi ne ru ta shi po an el or un is ex de re co pro tra ver
	gen mod str int fmt err ctx buf sql log net cfg env tmp run set get put add del
	map key val row col idx seq hash pin rec tok sum`)

// vocab builds vocabSize distinct terms. Rank 0 is the most frequent.
func vocab(r *rand.Rand) []string {
	seen := make(map[string]bool, vocabSize)
	out := make([]string, 0, vocabSize)
	for len(out) < vocabSize {
		var w string
		switch k := r.IntN(10); {
		case k < 6:
			n := 1 + r.IntN(4)
			var b strings.Builder
			for range n {
				b.WriteString(syllables[r.IntN(len(syllables))])
			}
			w = b.String()
		case k < 8:
			w = syllables[r.IntN(len(syllables))] + strings.ToUpper(syllables[r.IntN(len(syllables))][:1]) +
				syllables[r.IntN(len(syllables))] + syllables[r.IntN(len(syllables))]
		case k < 9:
			w = syllables[r.IntN(len(syllables))] + "_" + syllables[r.IntN(len(syllables))] + strconv.Itoa(r.IntN(100))
		default:
			w = strconv.Itoa(r.IntN(1_000_000))
		}
		if !seen[strings.ToLower(w)] {
			seen[strings.ToLower(w)] = true
			out = append(out, w)
		}
	}

	return out
}

type event struct {
	session string
	source  int64
	line    int64
	kind    string
	trust   int
	tool    any
	body    string
	payload bool
	ts      int64
}

// corpus generates the event stream.
type corpus struct {
	r      *rand.Rand
	zipf   *rand.Zipf
	words  []string
	perSes int64
	n      int64
}

func newCorpus(seed uint64) *corpus {
	r := rand.New(rand.NewPCG(seed, seed^0x9e3779b97f4a7c15))
	words := vocab(r)

	return &corpus{
		r:      r,
		zipf:   rand.NewZipf(r, 1.07, 2, vocabSize-1),
		words:  words,
		perSes: 5000,
	}
}

var kinds = []string{"user", "assistant", "tool_use", "tool_result", "system"}
var tools = []string{"Bash", "Read", "Edit", "Grep", "Write", "Glob"}

// bodyLen draws an event's inline text length: most events are short
// messages, some are medium, and a tail are payload previews capped
// at 2 KiB (larger payloads live in content-addressed files, §8.1).
func (c *corpus) bodyLen() (int, bool) {
	switch k := c.r.IntN(100); {
	case k < 60:
		return 40 + c.r.IntN(160), false
	case k < 90:
		return 200 + c.r.IntN(800), false
	default:
		return 2048, true
	}
}

func (c *corpus) next() event {
	i := c.n
	c.n++
	ses := i / c.perSes
	n, payload := c.bodyLen()
	var b strings.Builder
	b.Grow(n + 16)
	for b.Len() < n {
		if b.Len() > 0 {
			if c.r.IntN(12) == 0 {
				b.WriteString(". ")
			} else {
				b.WriteByte(' ')
			}
		}
		b.WriteString(c.words[c.zipf.Uint64()])
	}
	k := kinds[c.r.IntN(len(kinds))]
	var tool any
	if k == "tool_use" || k == "tool_result" {
		tool = tools[c.r.IntN(len(tools))]
	}

	return event{
		session: "ses-" + strconv.FormatInt(ses, 10),
		source:  ses,
		line:    i % c.perSes,
		kind:    k,
		trust:   c.r.IntN(3),
		tool:    tool,
		body:    b.String(),
		payload: payload,
		ts:      1_780_000_000_000 + i*37,
	}
}

// Query term bands by Zipf rank.
type band struct {
	name   string
	lo, hi int
}

var bands = []band{
	{"common", 20, 300},
	{"mid", 300, 5000},
	{"rare", 5000, vocabSize},
}

// query draws an FTS5 query of 1–3 terms from one band, quoting each
// term so identifiers with underscores parse as plain tokens.
func (c *corpus) query(r *rand.Rand, b band) string {
	n := 1 + r.IntN(3)
	terms := make([]string, n)
	for j := range terms {
		terms[j] = `"` + c.words[b.lo+r.IntN(b.hi-b.lo)] + `"`
	}

	return strings.Join(terms, " ")
}

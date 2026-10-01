package main

import (
	"context"
	"database/sql"
	"math/rand/v2"
	"path/filepath"
	"slices"
	"strings"
	"testing"
	"time"
)

func TestStatsTakesNearestRank(t *testing.T) {
	ds := make([]time.Duration, 300)
	for i := range ds {
		ds[i] = time.Duration(300-i) * time.Millisecond
	}

	got := stats(ds)

	// The p95 of 300 samples is the 285th smallest, the p50 the 150th.
	for _, want := range []string{"n=300", "p50=150ms", "p95=285ms", "p99=297ms", "max=300ms"} {
		if !strings.Contains(got, want) {
			t.Errorf("stats() = %q, want %s", got, want)
		}
	}
}

func TestStatsOfNothing(t *testing.T) {
	if got := stats(nil); got != "n=0" {
		t.Errorf("stats(nil) = %q, want n=0", got)
	}
}

func TestDSNEscapesURIMetacharacters(t *testing.T) {
	got := dsn("/tmp/a?b#c%d.db", "")

	if want := "file:/tmp/a%3fb%23c%25d.db?"; !strings.HasPrefix(got, want) {
		t.Errorf("dsn() = %q, want prefix %q", got, want)
	}
}

func TestOpenNamesTheFileItsPathSpells(t *testing.T) {
	for _, d := range []string{"modernc", "ncruces"} {
		t.Run(d, func(t *testing.T) {
			path := filepath.Join(t.TempDir(), "a?b.db")
			db, err := open(opts{driver: d, db: path})
			if err != nil {
				t.Fatal(err)
			}
			defer db.Close()
			if err := createSchema(context.Background(), db); err != nil {
				t.Fatal(err)
			}
			var file string
			if err := db.QueryRow(`SELECT file FROM pragma_database_list WHERE name = 'main'`).Scan(&file); err != nil {
				t.Fatal(err)
			}
			if file != path {
				t.Errorf("opened %q, want %q", file, path)
			}
		})
	}
}

func TestAddMetaAccumulates(t *testing.T) {
	ctx := context.Background()
	db, err := open(opts{driver: "ncruces", db: filepath.Join(t.TempDir(), "m.db")})
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()
	if err := createSchema(ctx, db); err != nil {
		t.Fatal(err)
	}
	for k, v := range map[string]int64{"events": 10, "text_bytes": 100} {
		if err := setMeta(ctx, db, k, v); err != nil {
			t.Fatal(err)
		}
	}

	if err := addMeta(ctx, db, 5, 50); err != nil {
		t.Fatal(err)
	}

	for k, want := range map[string]int64{"events": 15, "text_bytes": 150} {
		if got, err := getMeta(ctx, db, k); err != nil || got != want {
			t.Errorf("%s = %d, %v; want %d", k, got, err, want)
		}
	}
}

func TestAddMetaFailsWithoutCounters(t *testing.T) {
	ctx := context.Background()
	db, err := open(opts{driver: "ncruces", db: filepath.Join(t.TempDir(), "m.db")})
	if err != nil {
		t.Fatal(err)
	}
	defer db.Close()
	if err := createSchema(ctx, db); err != nil {
		t.Fatal(err)
	}

	if err := addMeta(ctx, db, 5, 50); err == nil {
		t.Error("addMeta on a store without counters succeeded, want an error")
	}
}

func TestCorpusStreamsShareTheStoreVocabulary(t *testing.T) {
	store, stream := newCorpus(1), newCorpusFrom(1, 2001)

	if !slices.Equal(store.words, stream.words) {
		t.Error("a stream seeded apart from the store draws from another vocabulary")
	}
	if store.next().body == stream.next().body {
		t.Error("a stream with its own seed repeats the store's events")
	}
}

func TestCorpusFromItsOwnSeedIsTheStoreCorpus(t *testing.T) {
	a, b := newCorpus(1), newCorpusFrom(1, 1)

	for range 100 {
		if ea, eb := a.next(), b.next(); ea != eb {
			t.Fatalf("newCorpusFrom(1, 1) drew %+v, newCorpus(1) drew %+v", eb, ea)
		}
	}
}

func TestFTS5ProbesFindTheirDocuments(t *testing.T) {
	for _, d := range []string{"modernc", "ncruces"} {
		t.Run(d, func(t *testing.T) {
			ctx := context.Background()
			db := probeStore(t, d)
			for _, p := range fts5Probes() {
				got, err := probeIDs(ctx, db, p.q)
				if err != nil || !slices.Equal(got, p.want) {
					t.Errorf("probe %s: got %v, %v; want %v", p.name, got, err, p.want)
				}
			}
		})
	}
}

func probeStore(t *testing.T, driver string) *sql.DB {
	t.Helper()
	ctx := context.Background()
	db, err := open(opts{driver: driver, db: filepath.Join(t.TempDir(), "f.db")})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { db.Close() })
	if err := createSchema(ctx, db); err != nil {
		t.Fatal(err)
	}
	if err := appendBodies(ctx, db, fts5Docs()); err != nil {
		t.Fatal(err)
	}

	return db
}

func TestSearchQueriesRefusesAnEmptyWindow(t *testing.T) {
	db := probeStore(t, "ncruces")

	_, _, err := searchQueries(context.Background(), db, newCorpus(1), rand.New(rand.NewPCG(1, 2)), 1, 9, 9)

	if err == nil {
		t.Error("searchQueries over an empty window succeeded, want an error")
	}
}

func TestSearchQueriesGivesUpWithoutAnchorTerms(t *testing.T) {
	ctx := context.Background()
	db := probeStore(t, "ncruces")
	if _, err := db.ExecContext(ctx, `UPDATE events SET body = 'zzqq'`); err != nil {
		t.Fatal(err)
	}

	_, _, err := searchQueries(ctx, db, newCorpus(1), rand.New(rand.NewPCG(1, 2)), 1, 0, 9)

	if err == nil {
		t.Error("searchQueries over events without corpus terms succeeded, want an error")
	}
}

func TestSearchRefusesRecentWithWindow(t *testing.T) {
	err := cmdSearch(context.Background(), opts{driver: "ncruces", recent: 2000, window: 10})

	if err == nil {
		t.Error("search with both -recent and -window succeeded, want an error")
	}
}

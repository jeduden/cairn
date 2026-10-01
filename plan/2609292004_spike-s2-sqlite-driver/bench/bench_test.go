package main

import (
	"context"
	"path/filepath"
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

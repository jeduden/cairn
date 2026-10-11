// Command ncruces is the smallest binary that links
// github.com/ncruces/go-sqlite3 with FTS5, for the import-closure and
// static-build checks. FTS5 is registered on each connection through
// driver.Open, the way the harness opens a store.
package main

import (
	"github.com/ncruces/go-sqlite3/driver"
	"github.com/ncruces/go-sqlite3/ext/fts5"
)

func main() {
	db, _ := driver.Open("file::memory:", fts5.Register)
	db.Exec(`CREATE VIRTUAL TABLE t USING fts5(x)`)
	db.Close()
}

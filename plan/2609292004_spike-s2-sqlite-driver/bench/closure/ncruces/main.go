// Command ncruces is the smallest binary that links
// github.com/ncruces/go-sqlite3 with FTS5, for the import-closure and
// static-build checks.
package main

import (
	"database/sql"

	"github.com/ncruces/go-sqlite3"
	_ "github.com/ncruces/go-sqlite3/driver"
	"github.com/ncruces/go-sqlite3/ext/fts5"
)

func main() {
	sqlite3.AutoExtension(fts5.Register)
	db, _ := sql.Open("sqlite3", "file::memory:")
	db.Exec(`CREATE VIRTUAL TABLE t USING fts5(x)`)
	db.Close()
}

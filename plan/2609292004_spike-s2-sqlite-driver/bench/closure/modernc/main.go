// Command modernc is the smallest binary that links modernc.org/sqlite
// with FTS5, for the import-closure and static-build checks.
package main

import (
	"database/sql"

	_ "modernc.org/sqlite"
)

func main() {
	db, _ := sql.Open("sqlite", "file::memory:")
	db.Exec(`CREATE VIRTUAL TABLE t USING fts5(x)`)
	db.Close()
}

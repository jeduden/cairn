package srs

import (
	"bufio"
	"bytes"
	"strings"
)

// Table is one pipe table read from a Markdown document: its header
// cells and its body rows, each row with the line it sits on.
type Table struct {
	Header []string
	Rows   []Row
}

// Row is one body row of a Table.
type Row struct {
	Cells []string
	Line  int
}

// Tables reads every pipe table in body. A table is a header row, a
// delimiter row of dashes, and the pipe rows that follow; lines inside
// a fenced code block never count, so a table quoted in an example is
// not a table.
func Tables(body []byte) []Table {
	var (
		out   []Table
		lines []string
	)
	sc := bufio.NewScanner(bytes.NewReader(body))
	sc.Buffer(make([]byte, 0, 64*1024), 1<<20)
	for sc.Scan() {
		lines = append(lines, sc.Text())
	}

	fence := ""
	for i := 0; i < len(lines); i++ {
		trimmed := strings.TrimSpace(lines[i])
		if f := FenceMarker(trimmed); f != "" {
			switch {
			case fence == "":
				fence = f
			case f[0] == fence[0] && len(f) >= len(fence):
				fence = ""
			}

			continue
		}
		if fence != "" || !isPipeRow(trimmed) || i+1 >= len(lines) || !isDelimiterRow(lines[i+1]) {
			continue
		}

		tbl := Table{Header: splitRow(trimmed)}
		i += 2
		for ; i < len(lines) && isPipeRow(strings.TrimSpace(lines[i])); i++ {
			tbl.Rows = append(tbl.Rows, Row{Cells: splitRow(strings.TrimSpace(lines[i])), Line: i + 1})
		}
		i--
		out = append(out, tbl)
	}

	return out
}

// FenceMarker returns the run of three or more backticks or tildes a
// line opens with, or "" when it is no fence. A fence closes only on a
// run of the same character at least as long as the one that opened
// it, so a four-backtick fence can quote a three-backtick one.
func FenceMarker(line string) string {
	if line == "" || (line[0] != '`' && line[0] != '~') {
		return ""
	}
	n := len(line) - len(strings.TrimLeft(line, line[:1]))
	if n < 3 {
		return ""
	}

	return line[:n]
}

func isPipeRow(line string) bool {
	return strings.HasPrefix(line, "|")
}

// isDelimiterRow reports whether line is a table's |---|:--:| row.
func isDelimiterRow(line string) bool {
	line = strings.TrimSpace(line)
	if !isPipeRow(line) {
		return false
	}
	cells := splitRow(line)
	for _, c := range cells {
		c = strings.Trim(c, ":")
		if c == "" || strings.Trim(c, "-") != "" {
			return false
		}
	}

	return len(cells) > 0
}

// splitRow splits a pipe row into trimmed cells. An escaped pipe (\|)
// stays inside its cell as a plain |.
func splitRow(line string) []string {
	line = strings.TrimPrefix(line, "|")
	if strings.HasSuffix(line, "|") && !strings.HasSuffix(line, `\|`) {
		line = strings.TrimSuffix(line, "|")
	}

	var (
		cells []string
		cur   strings.Builder
	)
	for i := 0; i < len(line); i++ {
		switch {
		case line[i] == '\\' && i+1 < len(line) && line[i+1] == '|':
			cur.WriteByte('|')
			i++
		case line[i] == '|':
			cells = append(cells, strings.TrimSpace(cur.String()))
			cur.Reset()
		default:
			cur.WriteByte(line[i])
		}
	}

	return append(cells, strings.TrimSpace(cur.String()))
}

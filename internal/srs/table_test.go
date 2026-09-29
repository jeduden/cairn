package srs

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTablesReadsHeaderAndRowsWithLines(t *testing.T) {
	body := []byte("intro\n\n| A | B |\n|---|:--:|\n| 1 | 2 |\n| 3 | 4 |\n\nafter\n")

	got := Tables(body)

	require.Len(t, got, 1)
	assert.Equal(t, []string{"A", "B"}, got[0].Header)
	assert.Equal(t, []Row{{Cells: []string{"1", "2"}, Line: 5}, {Cells: []string{"3", "4"}, Line: 6}}, got[0].Rows)
}

func TestTablesSkipsTablesInsideFences(t *testing.T) {
	body := []byte("```md\n| A |\n|---|\n| 1 |\n```\n~~~\n| B |\n|---|\n~~~\n| C |\n|---|\n| 2 |\n")

	got := Tables(body)

	require.Len(t, got, 1)
	assert.Equal(t, []string{"C"}, got[0].Header)
}

func TestTablesKeepsAFenceOpenUntilItsOwnMarker(t *testing.T) {
	body := []byte("````\n```\n| A |\n|---|\n````\n")

	assert.Empty(t, Tables(body))
}

func TestTablesNeedsADelimiterRow(t *testing.T) {
	assert.Empty(t, Tables([]byte("| A |\n| 1 |\n")))
	assert.Empty(t, Tables([]byte("| A |\n")))
	assert.Empty(t, Tables([]byte("| A |\nnot a row\n")))
}

func TestFenceMarker(t *testing.T) {
	assert.Equal(t, "```", fenceMarker("```go"))
	assert.Equal(t, "~~~~", fenceMarker("~~~~"))
	assert.Empty(t, fenceMarker("``inline``"))
	assert.Empty(t, fenceMarker(""))
	assert.Empty(t, fenceMarker("| a |"))
}

func TestIsDelimiterRow(t *testing.T) {
	assert.True(t, isDelimiterRow("| --- | :-: |"))
	assert.False(t, isDelimiterRow("| --- | x |"))
	assert.False(t, isDelimiterRow("| --- | : |"))
	assert.False(t, isDelimiterRow("---"))
}

func TestSplitRowUnescapesPipes(t *testing.T) {
	assert.Equal(t, []string{"a", "b|c", "d"}, splitRow(`| a | b\|c | d |`))
	assert.Equal(t, []string{"a", `x\|`}, splitRow(`| a | x\\|`))
	assert.Equal(t, []string{"a", "b"}, splitRow(`| a | b`))
}

package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"io/fs"
	"testing"

	"github.com/jeduden/cairn/internal/review"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

const head = "206b0a76a81abb510e53d87ef187e83b5aacbcbb"

func files(m map[string]string) func(string) ([]byte, error) {
	return func(name string) ([]byte, error) {
		body, ok := m[name]
		if !ok {
			return nil, fs.ErrNotExist
		}

		return []byte(body), nil
	}
}

func inputs() map[string]string {
	return map[string]string{
		"verdict.json": `{"verdict":"approve","summary":"Fine.","findings":[]}`,
		"checks.jsonl": `{"name":"CI","status":"completed","conclusion":"success"}`,
	}
}

func args(current string) []string {
	return []string{"-verdict", "verdict.json", "-checks", "checks.jsonl", "-reviewed", head, "-head", current}
}

func TestRunPrintsTheReviewToPost(t *testing.T) {
	var out, errOut bytes.Buffer
	code := run(args(head), &out, &errOut, files(inputs()))
	require.Equal(t, exitOK, code, errOut.String())
	var r review.Review
	require.NoError(t, json.Unmarshal(out.Bytes(), &r))
	assert.Equal(t, review.Approve, r.Event)
	assert.Equal(t, head, r.CommitID)
}

func TestRunPrintsNothingForAHeadThatMoved(t *testing.T) {
	var out, errOut bytes.Buffer
	code := run(args("490efa734617788d1dc1f831b26d0e23888e90fa"), &out, &errOut, files(inputs()))
	require.Equal(t, exitOK, code, errOut.String())
	assert.Empty(t, out.String())
	assert.Contains(t, errOut.String(), "moved past")
}

func TestRunFailsClosed(t *testing.T) {
	cases := map[string]struct {
		args  []string
		files map[string]string
		want  string
		code  int
	}{
		"an unknown flag":       {[]string{"-approve"}, inputs(), "flag provided but not defined", exitUsage},
		"a missing flag":        {args(head)[:6], inputs(), "-head is required", exitUsage},
		"a stray argument":      {append(args(head), "extra"), inputs(), "unexpected argument", exitUsage},
		"no verdict file":       {args(head), map[string]string{}, "verdict.json", exitFailure},
		"a malformed verdict":   {args(head), withFile("verdict.json", `{}`), "malformed verdict", exitFailure},
		"no checks file":        {args(head), without("checks.jsonl"), "checks.jsonl", exitFailure},
		"a malformed check":     {args(head), withFile("checks.jsonl", `nope`), "check line 1", exitFailure},
		"CI not passed at head": {args(head), withFile("checks.jsonl", ``), "CI has not passed", exitFailure},
	}
	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			var out, errOut bytes.Buffer
			code := run(c.args, &out, &errOut, files(c.files))
			assert.Equal(t, c.code, code)
			assert.Empty(t, out.String())
			assert.Contains(t, errOut.String(), c.want)
		})
	}
}

func TestRunReportsAFailedWrite(t *testing.T) {
	var errOut bytes.Buffer
	code := run(args(head), failingWriter{}, &errOut, files(inputs()))
	assert.Equal(t, exitFailure, code)
	assert.Contains(t, errOut.String(), "closed")
}

type failingWriter struct{}

func (failingWriter) Write([]byte) (int, error) { return 0, errors.New("closed") }

func withFile(name, body string) map[string]string {
	m := inputs()
	m[name] = body

	return m
}

func without(name string) map[string]string {
	m := inputs()
	delete(m, name)

	return m
}

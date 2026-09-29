package main

import (
	"bytes"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestRunWithNoArgsIsAUsageError(t *testing.T) {
	var out, errb bytes.Buffer

	code := run(nil, &out, &errb)

	assert.Equal(t, exitUsage, code)
	assert.Empty(t, out.String())
	assert.Contains(t, errb.String(), "usage: cairn")
}

func TestRunHelpPrintsUsageToStdout(t *testing.T) {
	for _, arg := range []string{"help", "-h", "--help"} {
		var out, errb bytes.Buffer

		code := run([]string{arg}, &out, &errb)

		assert.Equal(t, exitOK, code, arg)
		assert.Contains(t, out.String(), "usage: cairn", arg)
		assert.Empty(t, errb.String(), arg)
	}
}

func TestRunUnknownCommandIsAUsageError(t *testing.T) {
	var out, errb bytes.Buffer

	code := run([]string{"ingest"}, &out, &errb)

	assert.Equal(t, exitUsage, code)
	assert.Contains(t, errb.String(), `unknown command "ingest"`)
}

func TestRunVersionPrintsTheStampedVersion(t *testing.T) {
	var out, errb bytes.Buffer

	code := run([]string{"version"}, &out, &errb)

	assert.Equal(t, exitOK, code)
	assert.Equal(t, version+"\n", out.String())
}

func TestRunVersionJSONCarriesTheVersionKey(t *testing.T) {
	var out, errb bytes.Buffer

	code := run([]string{"version", "--json"}, &out, &errb)

	assert.Equal(t, exitOK, code)
	assert.JSONEq(t, `{"version":"dev"}`, out.String())
}

func TestRunVersionRejectsAnUnknownFlag(t *testing.T) {
	var out, errb bytes.Buffer

	code := run([]string{"version", "--yaml"}, &out, &errb)

	assert.Equal(t, exitUsage, code)
	assert.Contains(t, errb.String(), `unknown flag "--yaml"`)
}

// failingWriter refuses every write, standing in for a closed stdout.
type failingWriter struct{}

func (failingWriter) Write([]byte) (int, error) { return 0, errors.New("closed") }

func TestRunVersionJSONReportsAWriteFailure(t *testing.T) {
	var errb bytes.Buffer

	code := runVersion([]string{"--json"}, failingWriter{}, &errb)

	assert.Equal(t, exitFailure, code)
	assert.Contains(t, errb.String(), "closed")
}

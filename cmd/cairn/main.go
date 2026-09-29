// Command cairn is the Cairn binary: the hook entry point, the MCP
// server and the operator CLI, specified in docs/srs.
//
// Nothing past `version` is implemented yet. Every other command in
// the SRS's §9.5 is a pending scenario under features/, and lands with
// the plan that makes its scenario pass.
package main

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
)

// version is stamped by the release build with
// -ldflags "-X main.version=vX.Y.Z"; a local build reports "dev".
var version = "dev"

// Exit codes documented by SRS §9.5, shared by every command. The
// fourth, 3 for an integrity failure, arrives with `cairn verify`.
const (
	exitOK      = 0
	exitFailure = 1
	exitUsage   = 2
)

const usage = `usage: cairn <command> [flags]

commands:
  version [--json]   print the build version
  help               print this message
`

func main() {
	os.Exit(run(os.Args[1:], os.Stdout, os.Stderr))
}

// run dispatches one invocation and returns its exit code. It is main
// without the process: every test drives the CLI through it.
func run(args []string, stdout, stderr io.Writer) int {
	if len(args) == 0 {
		_, _ = io.WriteString(stderr, usage)

		return exitUsage
	}

	switch args[0] {
	case "version":
		return runVersion(args[1:], stdout, stderr)
	case "help", "-h", "--help":
		_, _ = io.WriteString(stdout, usage)

		return exitOK
	default:
		_, _ = fmt.Fprintf(stderr, "cairn: unknown command %q\n\n%s", args[0], usage)

		return exitUsage
	}
}

// runVersion prints the stamped version, as a bare line or, with
// --json, as {"version": "..."}.
func runVersion(args []string, stdout, stderr io.Writer) int {
	asJSON := false
	for _, a := range args {
		if a != "--json" {
			_, _ = fmt.Fprintf(stderr, "cairn version: unknown flag %q\n", a)

			return exitUsage
		}
		asJSON = true
	}

	if !asJSON {
		_, _ = fmt.Fprintln(stdout, version)

		return exitOK
	}
	if err := json.NewEncoder(stdout).Encode(struct {
		Version string `json:"version"`
	}{version}); err != nil {
		_, _ = fmt.Fprintf(stderr, "cairn version: %v\n", err)

		return exitFailure
	}

	return exitOK
}

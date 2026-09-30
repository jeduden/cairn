// Command review-gate decides the review the reviewer app posts on a
// pull request (ENG-28). The review workflow runs it from main's copy
// of the repository, in the one job that holds the app's key, after
// the reviewing agent wrote its verdict. It prints the review as the
// JSON GitHub's create-review endpoint takes, prints nothing when the
// pull request moved past the reviewed head, and exits non-zero on
// anything it cannot decide, so the job fails and posts nothing.
//
// It reads files and writes stdout only; the workflow fetches the
// checks and posts the review. It is CI tooling and never ships.
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"os"

	"github.com/jeduden/cairn/internal/review"
)

const (
	exitOK      = 0
	exitFailure = 1
	exitUsage   = 2
)

func main() {
	os.Exit(run(os.Args[1:], os.Stdout, os.Stderr, os.ReadFile))
}

// run is main without the process: it reads the verdict and checks
// files through readFile and returns the exit code.
func run(args []string, stdout, stderr io.Writer, readFile func(string) ([]byte, error)) int {
	fl := flag.NewFlagSet("review-gate", flag.ContinueOnError)
	fl.SetOutput(stderr)
	verdictFile := fl.String("verdict", "", "the agent's verdict, as JSON")
	checksFile := fl.String("checks", "", "the reviewed head's check runs, one JSON object per line")
	reviewed := fl.String("reviewed", "", "the head commit the agent reviewed")
	head := fl.String("head", "", "the pull request's head commit now")
	if err := fl.Parse(args); err != nil {
		return exitUsage
	}
	if fl.NArg() > 0 {
		_, _ = fmt.Fprintf(stderr, "review-gate: unexpected argument %q\n", fl.Arg(0))

		return exitUsage
	}
	for _, f := range []struct{ name, value string }{
		{"verdict", *verdictFile}, {"checks", *checksFile}, {"reviewed", *reviewed}, {"head", *head},
	} {
		if f.value == "" {
			_, _ = fmt.Fprintf(stderr, "review-gate: -%s is required\n", f.name)

			return exitUsage
		}
	}
	in, err := read(*verdictFile, *checksFile, readFile)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "review-gate: %v\n", err)

		return exitFailure
	}
	in.Reviewed, in.Head = *reviewed, *head

	return decide(in, stdout, stderr)
}

// read loads the verdict and the checks.
func read(verdictFile, checksFile string, readFile func(string) ([]byte, error)) (review.Input, error) {
	body, err := readFile(verdictFile)
	if err != nil {
		return review.Input{}, fmt.Errorf("read %s: %w", verdictFile, err)
	}
	v, err := review.ParseVerdict(body)
	if err != nil {
		return review.Input{}, err
	}
	body, err = readFile(checksFile)
	if err != nil {
		return review.Input{}, fmt.Errorf("read %s: %w", checksFile, err)
	}
	checks, err := review.ParseChecks(body)
	if err != nil {
		return review.Input{}, err
	}

	return review.Input{Verdict: v, Checks: checks}, nil
}

// decide prints the review the gate decides on, or says why it prints
// none.
func decide(in review.Input, stdout, stderr io.Writer) int {
	r, post, err := review.Decide(in)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "review-gate: %v\n", err)

		return exitFailure
	}
	if !post {
		_, _ = fmt.Fprintf(stderr, "review-gate: the pull request moved past %s; its newer run reviews it\n",
			in.Reviewed)

		return exitOK
	}
	if err := json.NewEncoder(stdout).Encode(r); err != nil {
		_, _ = fmt.Fprintf(stderr, "review-gate: %v\n", err)

		return exitFailure
	}

	return exitOK
}

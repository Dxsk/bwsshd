// Command bwsshd keeps an ssh_config in sync with the SSH keys held by the
// Bitwarden desktop SSH agent. The implementation lives in internal/agent; this
// file is only the entry point so the module installs as `bwsshd`.
package main

import "github.com/Dxsk/bwsshd/internal/agent"

// version is injected at release time by goreleaser (-ldflags -X main.version).
// It is intentionally empty by default: nobody edits a version by hand. For a
// plain `go build`/`go install` the agent package falls back to the data Go
// embeds itself.
var version = ""

func main() { agent.Run(version) }

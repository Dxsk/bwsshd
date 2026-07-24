//go:build !windows

package agent

import "testing"

// On unix, auto-discovery ends with the Snap socket path derived from home.
func TestDefaultAgentPathsUnix(t *testing.T) {
	got := candidateSocks("", "/home/u")
	want := "/home/u/snap/bitwarden/current/.bitwarden-ssh-agent.sock"
	if got[len(got)-1] != want {
		t.Errorf("last default should be the snap socket: got %v", got)
	}
}

// On unix an absolute Include path works directly.
func TestFormatIncludeUnix(t *testing.T) {
	if got := formatInclude("/home/u/.ssh", "/home/u/.ssh/config.automatic.bw"); got != "/home/u/.ssh/config.automatic.bw" {
		t.Errorf("want absolute include, got %q", got)
	}
}

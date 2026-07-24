//go:build windows

package agent

import (
	"path/filepath"
	"strings"
	"testing"

	"golang.org/x/crypto/ssh/agent"
)

// On Windows, auto-discovery resolves to the Bitwarden named pipe, which
// registers as the standard OpenSSH agent pipe.
func TestDefaultAgentPathsWindows(t *testing.T) {
	got := candidateSocks("", `C:\Users\u`)
	want := `\\.\pipe\openssh-ssh-agent`
	if got[len(got)-1] != want {
		t.Errorf("last default should be the openssh-ssh-agent pipe: got %v", got)
	}
}

// The generated block must use forward slashes: OpenSSH on Windows accepts
// //./pipe/... and C:/... but treats backslashes as escapes. A backslash in the
// Host block would silently break the pipe path or the key path.
func TestGenerateForwardSlashesWindows(t *testing.T) {
	dir := t.TempDir()
	out := filepath.Join(dir, "config.bw")
	keydir := filepath.Join(dir, "keys")
	if err := generate([]*agent.Key{key(t, "debian@host.fr")}, out, keydir, `\\.\pipe\openssh-ssh-agent`); err != nil {
		t.Fatal(err)
	}
	cfg := read(t, out)
	if !strings.Contains(cfg, "IdentityAgent //./pipe/openssh-ssh-agent") {
		t.Errorf("pipe not forward-slashed:\n%s", cfg)
	}
	for _, l := range strings.Split(cfg, "\n") {
		if strings.HasPrefix(strings.TrimSpace(l), "Identity") && strings.Contains(l, `\`) {
			t.Errorf("Identity line still contains a backslash: %q", l)
		}
	}
}

// A C:\ Include is mangled by OpenSSH on Windows; it must be written relative to
// the config directory with forward slashes.
func TestFormatIncludeWindows(t *testing.T) {
	got := formatInclude(`C:\Users\u\.ssh`, `C:\Users\u\.ssh\config.automatic.bw`)
	if got != "config.automatic.bw" {
		t.Errorf("want config-relative forward-slash include, got %q", got)
	}
}

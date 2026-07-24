//go:build windows

// Windows-specific glue: the Bitwarden desktop agent is a named pipe, not a unix
// socket, and there is no root user to refuse.
package agent

import (
	"net"
	"path/filepath"
	"strings"

	"github.com/Microsoft/go-winio"
)

// checkNotRoot is a no-op on Windows: there is no euid, and files are written
// under the user's own profile regardless.
func checkNotRoot() error { return nil }

// defaultAgentPaths returns the named pipe Bitwarden exposes on Windows. The
// desktop app registers itself as the standard OpenSSH agent pipe, so it is a
// drop-in replacement for Windows' own ssh-agent.
func defaultAgentPaths(home string) []string {
	return []string{`\\.\pipe\openssh-ssh-agent`}
}

// dialAgent connects to the agent at path. A `\\`-prefixed path is a named pipe
// (the Bitwarden agent); anything else is treated as a unix-domain socket, which
// Windows 10+ supports and which lets -sock point at one for testing.
func dialAgent(path string) (net.Conn, error) {
	if strings.HasPrefix(path, `\\`) {
		return winio.DialPipe(path, nil)
	}
	return net.Dial("unix", path)
}

// formatInclude returns the text to write after "Include ". OpenSSH on Windows
// treats a token that does not start with '/' as relative to ~/.ssh and mangles
// backslashes as glob escapes, so an absolute C:\ path never matches. We emit a
// forward-slash path relative to the config directory (which is ~/.ssh in the
// default layout), which OpenSSH resolves correctly. If the generated file lives
// outside that directory we fall back to an absolute forward-slash path.
func formatInclude(sshConfigDir, out string) string {
	if rel, err := filepath.Rel(sshConfigDir, out); err == nil && !strings.HasPrefix(rel, "..") {
		return filepath.ToSlash(rel)
	}
	if abs, err := filepath.Abs(out); err == nil {
		return filepath.ToSlash(abs)
	}
	return filepath.ToSlash(out)
}

//go:build !windows

// Unix-specific glue: the Bitwarden desktop agent is a unix-domain socket, and
// running as root would write into the user's ~/.ssh as root.
package agent

import (
	"fmt"
	"net"
	"os"
	"path/filepath"
)

// checkNotRoot refuses to run as root: keys and config would be written owned by
// root in the user's ~/.ssh, defeating the whole point.
func checkNotRoot() error {
	if os.Geteuid() == 0 {
		return fmt.Errorf("refusing to run as root; run as your own user")
	}
	return nil
}

// defaultAgentPaths lists the Bitwarden desktop agent sockets to try, in order,
// across native, Flatpak and Snap installs. The socket is created by the desktop
// client, so a Vaultwarden backend uses the same paths.
func defaultAgentPaths(home string) []string {
	return []string{
		filepath.Join(home, ".bitwarden-ssh-agent.sock"),                                     // native / deb / rpm
		filepath.Join(home, ".var/app/com.bitwarden.desktop/data/.bitwarden-ssh-agent.sock"), // flatpak
		filepath.Join(home, "snap/bitwarden/current/.bitwarden-ssh-agent.sock"),              // snap
	}
}

// dialAgent connects to the agent at path. On unix that is always a
// unix-domain socket.
func dialAgent(path string) (net.Conn, error) {
	return net.Dial("unix", path)
}

// formatInclude returns the text to write after "Include ". On unix an absolute
// path works directly (it starts with '/'), so we keep the current behavior.
func formatInclude(sshConfigDir, out string) string {
	if abs, err := filepath.Abs(out); err == nil {
		return abs
	}
	return out
}

//go:build !windows

package agent

import (
	"crypto/ed25519"
	"crypto/rand"
	"net"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"golang.org/x/crypto/ssh/agent"
)

// serveAgent starts an in-process SSH agent on a unix socket holding one
// ed25519 key with the given comment, and returns the socket path. This mirrors
// the Bitwarden transport on Linux/macOS; on Windows the transport is a named
// pipe (see platform_windows.go) and is exercised against the real agent.
func serveAgent(t *testing.T, comment string) string {
	t.Helper()
	_, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatal(err)
	}
	kr := agent.NewKeyring()
	if err := kr.Add(agent.AddedKey{PrivateKey: priv, Comment: comment}); err != nil {
		t.Fatal(err)
	}
	sock := filepath.Join(t.TempDir(), "agent.sock")
	ln, err := net.Listen("unix", sock)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { ln.Close() })
	go func() {
		for {
			c, err := ln.Accept()
			if err != nil {
				return
			}
			go agent.ServeAgent(kr, c)
		}
	}()
	return sock
}

func TestSyncPresentThenAbsent(t *testing.T) {
	dir := t.TempDir()
	out := filepath.Join(dir, "config.bw")
	keydir := filepath.Join(dir, "keys")
	sock := serveAgent(t, "debian@host.fr:2222")

	state, err := sync([]string{sock}, out, keydir, "")
	if err != nil {
		t.Fatal(err)
	}
	if state == stateAbsent {
		t.Fatal("expected present state")
	}
	if !strings.Contains(read(t, out), "Host host.fr") {
		t.Error("sync did not generate the host block")
	}

	// Same state -> no-op, returns unchanged.
	if s, err := sync([]string{sock}, out, keydir, state); err != nil || s != state {
		t.Errorf("idempotent sync: state=%q err=%v", s, err)
	}

	// No reachable socket -> clean: keydir removed, config blanked.
	gone := filepath.Join(dir, "nope.sock")
	s, err := sync([]string{gone}, out, keydir, state)
	if err != nil {
		t.Fatal(err)
	}
	if s != stateAbsent {
		t.Errorf("want absent, got %q", s)
	}
	if _, err := os.Stat(keydir); !os.IsNotExist(err) {
		t.Error("clean did not remove keydir on absent agent")
	}
	if !strings.Contains(read(t, out), "socket absent") {
		t.Error("config not blanked on absent agent")
	}
}

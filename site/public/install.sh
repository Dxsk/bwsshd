#!/bin/sh
# bwsshd installer for Linux and macOS.
#
# Downloads the latest release for this machine, checks it against the
# published checksums, installs the binary into ~/.local/bin and sets it up to
# run at login (systemd user service on Linux, LaunchAgent on macOS).
#
#   BWSSHD_BIN_DIR=/path     install the binary somewhere else
#   BWSSHD_NO_SERVICE=1      only install the binary
set -eu

REPO="Dxsk/bwsshd"
BIN_DIR="${BWSSHD_BIN_DIR:-$HOME/.local/bin}"

say() { printf 'bwsshd: %s\n' "$*"; }
die() {
    say "$*" >&2
    exit 1
}

[ "$(id -u)" -ne 0 ] || die "run this as your own user, not root"

case "$(uname -s)" in
Linux) os=linux ;;
Darwin) os=darwin ;;
*) die "unsupported system $(uname -s); see https://github.com/$REPO/releases" ;;
esac
case "$(uname -m)" in
x86_64 | amd64) arch=amd64 ;;
aarch64 | arm64) arch=arm64 ;;
*) die "unsupported CPU $(uname -m); see https://github.com/$REPO/releases" ;;
esac

if command -v curl >/dev/null 2>&1; then
    fetch() { curl -fsSL "$1"; }
elif command -v wget >/dev/null 2>&1; then
    fetch() { wget -qO- "$1"; }
else
    die "curl or wget is required"
fi

tag=$(fetch "https://api.github.com/repos/$REPO/releases/latest" |
    sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)
[ -n "$tag" ] || die "could not find the latest release"
name="bwsshd_${tag#v}_${os}_${arch}"
base="https://github.com/$REPO/releases/download/$tag"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

say "downloading $name"
fetch "$base/$name.tar.gz" >"$tmp/$name.tar.gz" || die "no $name.tar.gz in release $tag"

if fetch "$base/checksums.txt" >"$tmp/checksums.txt" 2>/dev/null; then
    want=$(awk -v f="$name.tar.gz" '$2 == f { print $1 }' "$tmp/checksums.txt")
    if command -v sha256sum >/dev/null 2>&1; then
        got=$(sha256sum "$tmp/$name.tar.gz" | awk '{ print $1 }')
    else
        got=$(shasum -a 256 "$tmp/$name.tar.gz" | awk '{ print $1 }')
    fi
    [ -z "$want" ] || [ "$want" = "$got" ] || die "checksum mismatch for $name.tar.gz"
fi

mkdir -p "$tmp/x"
tar -xzf "$tmp/$name.tar.gz" -C "$tmp/x"
bin=$(find "$tmp/x" -type f -name bwsshd | head -n 1)
[ -n "$bin" ] || die "the archive has no bwsshd binary"

mkdir -p "$BIN_DIR"
cp "$bin" "$BIN_DIR/bwsshd.new"
chmod 755 "$BIN_DIR/bwsshd.new"
mv -f "$BIN_DIR/bwsshd.new" "$BIN_DIR/bwsshd"
say "installed $("$BIN_DIR/bwsshd" --version) to $BIN_DIR/bwsshd"

case ":$PATH:" in
*":$BIN_DIR:"*) ;;
*) say "note: $BIN_DIR is not on your PATH" ;;
esac

if [ -n "${BWSSHD_NO_SERVICE:-}" ]; then
    say "skipping the login service (BWSSHD_NO_SERVICE is set); run: bwsshd --watch"
    exit 0
fi

if [ "$os" = linux ]; then
    if ! command -v systemctl >/dev/null 2>&1 || ! systemctl --user show-environment >/dev/null 2>&1; then
        say "no systemd user session; start it yourself with: bwsshd --watch"
        exit 0
    fi
    unit_dir="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
    mkdir -p "$unit_dir"
    cat >"$unit_dir/bwsshd.service" <<EOF
[Unit]
Description=bwsshd - keep ssh_config in sync with the Bitwarden SSH agent
After=default.target

[Service]
ExecStart=$BIN_DIR/bwsshd --watch
Restart=on-failure
RestartSec=5
NoNewPrivileges=true
UMask=0077

[Install]
WantedBy=default.target
EOF
    systemctl --user daemon-reload
    systemctl --user enable bwsshd.service >/dev/null 2>&1
    systemctl --user restart bwsshd.service
    say "running as a systemd user service; logs: journalctl --user -u bwsshd -f"
else
    label="fr.infrasouveraine.bwsshd"
    plist="$HOME/Library/LaunchAgents/$label.plist"
    log="$HOME/Library/Logs/bwsshd.log"
    mkdir -p "$HOME/Library/LaunchAgents" "$HOME/Library/Logs"
    cat >"$plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>$label</string>
    <key>ProgramArguments</key>
    <array>
        <string>$BIN_DIR/bwsshd</string>
        <string>--watch</string>
        <string>--tray</string>
        <string>--logfile</string>
        <string>$log</string>
    </array>
    <key>RunAtLoad</key><true/>
    <!-- Restart on crash, but not after Quit from the menu bar icon. -->
    <key>KeepAlive</key><dict><key>SuccessfulExit</key><false/></dict>
</dict>
</plist>
EOF
    launchctl bootout "gui/$(id -u)/$label" >/dev/null 2>&1 || true
    launchctl bootstrap "gui/$(id -u)" "$plist"
    say "running as a LaunchAgent; logs: tail -f $log"
fi

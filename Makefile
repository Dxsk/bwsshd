BIN := dist/sshwarden
PREFIX := $(HOME)/.local
UNITDIR := $(HOME)/.config/systemd/user

.PHONY: build test clean install uninstall

build:
	CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o $(BIN) .

test:
	go test ./...

clean:
	rm -f $(BIN)

install: build
	install -Dm755 $(BIN) $(PREFIX)/bin/sshwarden
	install -Dm644 systemd/sshwarden.service $(UNITDIR)/sshwarden.service
	systemctl --user daemon-reload
	systemctl --user enable --now sshwarden.service
	@echo "logs: journalctl --user -u sshwarden -f"

uninstall:
	-systemctl --user disable --now sshwarden.service
	rm -f $(PREFIX)/bin/sshwarden $(UNITDIR)/sshwarden.service
	systemctl --user daemon-reload

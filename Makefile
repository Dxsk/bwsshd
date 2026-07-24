# Cross-platform build/test/run. Windows adds a .exe suffix and runs tests with
# a project-local temp dir, because some Windows app-control policies block
# executing freshly built test binaries out of %TEMP%.
ifeq ($(OS),Windows_NT)
	EXE := .exe
else
	EXE :=
endif

BIN := dist/bwsshd$(EXE)
GOTMP := $(CURDIR)/dist/.gotmp
PREFIX := $(HOME)/.local
UNITDIR := $(HOME)/.config/systemd/user

.PHONY: build test run clean install uninstall

build:
	CGO_ENABLED=0 go build -trimpath -ldflags="-s -w" -o $(BIN) .

test:
	@mkdir -p "$(GOTMP)"
	GOTMPDIR="$(GOTMP)" go test ./...

run: build
	./$(BIN)

clean:
	rm -f $(BIN)
	rm -rf $(GOTMP)

# install / uninstall are Linux-only: they wire up the systemd --user service.
install: build
	install -Dm755 $(BIN) $(PREFIX)/bin/bwsshd
	install -Dm644 systemd/bwsshd.service $(UNITDIR)/bwsshd.service
	systemctl --user daemon-reload
	systemctl --user enable --now bwsshd.service
	@echo "logs: journalctl --user -u bwsshd -f"

uninstall:
	-systemctl --user disable --now bwsshd.service
	rm -f $(PREFIX)/bin/bwsshd $(UNITDIR)/bwsshd.service
	systemctl --user daemon-reload

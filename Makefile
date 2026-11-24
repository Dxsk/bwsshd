# Cross-platform build/test/run. On Windows run it from Git Bash (POSIX shell);
# the binary gets a .exe suffix.
ifeq ($(OS),Windows_NT)
	EXE := .exe
else
	EXE :=
endif

BIN := dist/bwsshd$(EXE)
PREFIX := $(HOME)/.local
UNITDIR := $(HOME)/.config/systemd/user

.PHONY: build test lint run clean install uninstall

build:
	cargo build --release --locked
	@mkdir -p dist
	cp target/release/bwsshd$(EXE) $(BIN)

test:
	cargo test --locked

lint:
	cargo fmt --check
	cargo clippy --all-targets --locked -- -D warnings

run: build
	./$(BIN)

clean:
	cargo clean
	rm -f $(BIN)

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

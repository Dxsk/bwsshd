# Cross-platform build/test/run. On Windows run it from Git Bash (POSIX shell);
# the binary gets a .exe suffix.
ifeq ($(OS),Windows_NT)
	EXE := .exe
else
	EXE :=
endif

BIN := dist/bwsshd$(EXE)
PREFIX := $(HOME)/.local

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

# Copies the binary to $(PREFIX)/bin and starts it at login (see `bwsshd install`).
install: build
	install -Dm755 $(BIN) $(PREFIX)/bin/bwsshd$(EXE)
	$(PREFIX)/bin/bwsshd$(EXE) install

uninstall:
	-$(PREFIX)/bin/bwsshd$(EXE) uninstall
	rm -f $(PREFIX)/bin/bwsshd$(EXE)

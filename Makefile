APP_NAME := tailtui

.PHONY: help build run test check fmt fmt-check clippy clean specs install-wsl

help:
	@echo "Available targets:"
	@echo "  make build      - Compile debug binary"
	@echo "  make run        - Run the app"
	@echo "  make test       - Run tests"
	@echo "  make check      - Fast compile checks"
	@echo "  make fmt        - Format code"
	@echo "  make fmt-check  - Verify formatting"
	@echo "  make clippy     - Lint with clippy (warnings denied)"
	@echo "  make clean      - Remove build artifacts"
	@echo "  make specs      - List spec files"
	@echo "  make install-wsl - WSL2: Windows tailscale CLI + cargo install tailtui"

build:
	cargo build

run:
	cargo run

test:
	cargo test

check:
	cargo check

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean

specs:
	@ls -1 specs

install-wsl:
	bash scripts/install-wsl.sh

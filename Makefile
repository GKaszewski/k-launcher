.PHONY: build build-egui dev check test fmt run run-egui install install-egui clean

RELEASE_BIN := target/release/k-launcher
EGUI_BIN    := target/release/k-launcher-egui
INSTALL_DIR := $(HOME)/.local/bin

build:
	cargo build --release

build-egui:
	cargo build --release -p k-launcher --features egui --bin k-launcher-egui

dev:
	RUST_LOG=debug cargo run

check:
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings
	cargo test --workspace

test:
	cargo test --workspace

fmt:
	cargo fmt --all

run:
	cargo run --release

run-egui:
	cargo run --release -p k-launcher --features egui --bin k-launcher-egui

install: build
	install -Dm755 $(RELEASE_BIN) $(INSTALL_DIR)/k-launcher

install-egui: build-egui
	install -Dm755 $(EGUI_BIN) $(INSTALL_DIR)/k-launcher-egui

clean:
	cargo clean

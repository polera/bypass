.DEFAULT_GOAL := help

.PHONY: help build release check checks ensure-check-tools test lint fmt fmt-check clean run 

CARGO_ENV := source "$$HOME/.cargo/env" &&

help:
	@echo "Available targets:"
	@awk -F: '/^[a-zA-Z0-9][a-zA-Z0-9_.-]*:/{print " - " $$1}' Makefile

build:
	$(CARGO_ENV) cargo build --tests

release:
	$(CARGO_ENV) cargo build --tests --release

check:
	$(CARGO_ENV) cargo check --tests

ensure-check-tools:
	$(CARGO_ENV) if ! cargo clippy --version >/dev/null 2>&1; then \
		echo "clippy is not installed; bootstrapping via rustup..."; \
		rustup component add clippy; \
	fi
	$(CARGO_ENV) if ! command -v cargo-audit >/dev/null 2>&1; then \
		echo "cargo-audit is not installed; bootstrapping via cargo install..."; \
		cargo install --locked cargo-audit; \
	fi

checks: ensure-check-tools
	$(CARGO_ENV) cargo clippy --tests -- -D warnings
	$(CARGO_ENV) cargo audit

test:
	$(CARGO_ENV) cargo test

lint:
	$(CARGO_ENV) cargo clippy --tests -- -D warnings

fmt:
	$(CARGO_ENV) cargo fmt

fmt-check:
	$(CARGO_ENV) cargo fmt -- --check

clean:
	$(CARGO_ENV) cargo clean

run:
	$(CARGO_ENV) cargo run

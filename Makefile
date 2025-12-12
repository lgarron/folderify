
.PHONY: build
build:
	cargo build --release

.PHONY: check
check: lint test build

.PHONY: lint
lint: setup-js
	cargo clippy -- --deny warnings
	cargo fmt --check
	bun x readme-cli-help check
	bun x @biomejs/biome check

.PHONY: format
format: setup-js
	cargo clippy
	cargo fmt
	bun x readme-cli-help update
	bun x @biomejs/biome check --write

.PHONY: setup
setup: setup-js

.PHONY: setup-js
setup-js:
	bun install --frozen-lockfile

.PHONY: test
test: cargo-test test-behaviour

.PHONY: cargo-test
cargo-test:
	cargo test

.PHONY: test-behaviour
test-behaviour: setup-js
	bun test --timeout 15000

.PHONY: publish
publish:
	# `--no-verify` is a workaround for https://github.com/rust-lang/cargo/issues/8407
	cargo publish --no-verify

.PHONY: install
install:
	cargo install --path .

.PHONY: uninstall
	cargo uninstall folderify

.PHONY: clean
clean:

.PHONY: reset
reset:
	rm -rf ./target

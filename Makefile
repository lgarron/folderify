.PHONY: build
build:
	cargo build --release

.PHONY: lint
lint:
	cargo clippy -- --deny warnings
	cargo fmt --check

.PHONY: format
format:
	cargo clippy
	cargo fmt --check

.PHONY: test
test: test-behaviour lint check-readme-cli-help

.PHONY: test-behaviour
test-behaviour:
	./test/test-behaviour.sh

.PHONY: publish
publish:
	cargo publish

.PHONY: install
install:
	cargo install --path .

.PHONY: uninstall
	cargo uninstall folderify

.PHONY: clean
clean:

.PHONY: readme-cli-help
readme-cli-help:
	bun x readme-cli-help "cargo run -- --help"

.PHONY: check-readme-cli-help
check-readme-cli-help:
	bun x readme-cli-help --check-only "cargo run -- --help"

.PHONY: reset
reset:
	rm -rf ./target

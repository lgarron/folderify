.PHONY: build
build:
	cargo build --release

.PHONY: lint
lint:
	cargo clippy
	cargo fmt --check

.PHONY: format
format:
	cargo fmt

.PHONY: test
test:
	./test/test.sh

.PHONY: publish
publish:
	cargo publish

.PHONY: install
install:
	cargo install --path .

.PHONY: uninstall
	cargo uninstall folderify

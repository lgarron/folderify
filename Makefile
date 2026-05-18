
.PHONY: build
build:
	cargo build --release

.PHONY: check
check: lint test build

.PHONY: lint
lint: lint-clippy lint-cargo-fmt lint-readme-cli-help lint-biome lint-typescript

.PHONY: lint-clippy
lint-clippy: setup-js
	cargo clippy -- --deny warnings

.PHONY: lint-cargo-fmt
lint-cargo-fmt: setup-js
	cargo fmt --check

.PHONY: lint-readme-cli-help
lint-readme-cli-help: setup-js
	bun x -- bun-dx --package readme-cli-help readme-cli-help -- check

.PHONY: lint-biome
lint-biome: setup-js
	bun x -- bun-dx --package @biomejs/biome biome -- check

.PHONY: lint-typescript
lint-typescript:
	bun x -- bun-dx --package typescript tsc -- --project ./tsconfig.json

.PHONY: format
format: setup-js
	cargo clippy
	cargo fmt
	bun x -- bun-dx --package readme-cli-help readme-cli-help -- update
	bun x -- bun-dx --package @biomejs/biome biome -- check --write

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

RM_RF = bun -e 'process.argv.slice(1).map(p => process.getBuiltinModule("node:fs").rmSync(p, {recursive: true, force: true, maxRetries: 5}))' --

.PHONY: clean
clean:
	@ # no-op for now

.PHONY: reset
reset: clean
	${RM_RF} ./node_modules/ ./target/

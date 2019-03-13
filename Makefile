build:
	@cargo build
.PHONY: build

watch:
	@cargo watch
.PHONY: watch

watch-docs:
	@cargo watch build "doc --no-deps"
.PHONY: watch-docs

format:
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt
.PHONY: format

check-format:
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt -- --check
.PHONY: check-format

test:
	@cargo test
.PHONY: test

check: check-format test
.PHONY: check

docs: build
	@cargo doc --no-deps
.PHONY: docs

upload-docs: docs
	@./upload-docs.sh
.PHONY: upload-docs

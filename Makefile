build:
	@cargo build

watch:
	@cargo watch build "doc --no-deps"

test:
	@cargo test

docs: build
	@cargo doc --no-deps

upload-docs: docs
	@./upload-docs.sh

.PHONY: build test docs upload-docs

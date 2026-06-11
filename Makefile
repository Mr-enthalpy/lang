.PHONY: test check fmt

test:
	cargo test

check:
	cargo check --workspace

fmt:
	cargo fmt --all

.PHONY: fmt test integration-tests clippy
fmt:
	cargo fmt --all

test:
	cargo test --all-features --all

integration-tests:
	# TODO
	cargo test

clippy:
	cargo clippy --all-targets --all-features --tests --benches -- -D warnings

publish:
	cargo publish --allow-dirty -p polars-arrow
	sleep 20

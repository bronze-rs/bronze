.PHONY: fmt test integration-tests clippy
fmt:
	cargo fmt --all

test:
	cargo test --all-features --all
	cargo test --all --no-default-features
	cargo test --all --no-default-features --features async

integration-tests:
	# TODO
	cargo test

clippy:
	cargo clippy --all-targets --all-features --tests --benches -- -D warnings
	cargo clippy --all-targets --all-features --tests --benches --no-default-features -- -D warnings
	cargo clippy --all-targets --all-features --tests --benches --no-default-features --features async -- -D warnings

publish:
	cargo publish --allow-dirty --registry crates-io -p bronzeflow-utils
	sleep 20

	cargo publish --allow-dirty --registry crates-io -p bronzeflow-time
	sleep 20

	cargo publish --allow-dirty --registry crates-io -p bronzeflow-core
	sleep 20

	cargo publish --allow-dirty --registry crates-io -p bronzeflow
	sleep 20

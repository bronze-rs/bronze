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
	cargo publish --allow-dirty --registry crates-io -p bronzeflow-utils
	sleep 20

	cargo publish --allow-dirty --registry crates-io -p bronzeflow-time
	sleep 20

	cargo publish --allow-dirty --registry crates-io -p bronzeflow-core
	sleep 20

	cargo publish --allow-dirty --registry crates-io -p bronzeflow
	sleep 20

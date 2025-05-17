include vars.env

nightly = +$(subst ",,${RUST_TOOLCHAIN_NIGHTLY})

clippy-%:
	cargo $(nightly) clippy --manifest-path $(subst -,/,$*)/Cargo.toml

format-%:
	cargo $(nightly) fmt --check --manifest-path $(subst -,/,$*)/Cargo.toml

format-%-fix:
	cargo $(nightly) fmt --manifest-path $(subst -,/,$*)/Cargo.toml

features-%:
	cargo $(nightly) hack check --feature-powerset --all-targets --manifest-path $(subst -,/,$*)/Cargo.toml

publish-%:
	./scripts/publish-rust.sh $(subst -,/,$*)

lint-docs-%:
	RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo $(nightly) doc --all-features --no-deps --manifest-path $(subst -,/,$*)/Cargo.toml

lint-features-%:
	cargo $(nightly) hack check --feature-powerset --all-targets --manifest-path $(subst -,/,$*)/Cargo.toml

build-%:
	cargo build --manifest-path $(subst -,/,$*)/Cargo.toml

build-program:
	cargo build-sbf --manifest-path program/Cargo.toml --features bpf-entrypoint

test-%:
	cargo $(nightly) test --manifest-path $(subst -,/,$*)/Cargo.toml

test-program:
	cargo test-sbf --manifest-path program/Cargo.toml --features bpf-entrypoint

bench-program-compute-units:
	cargo bench --manifest-path program/Cargo.toml

conformance:
	./scripts/conformance.sh

format-js:
	cd ./clients/js && pnpm install && pnpm format

lint-js:
	cd ./clients/js && pnpm install && pnpm lint

test-js:
	./scripts/restart-test-validator.sh
	cd ./clients/js && pnpm install && pnpm build && pnpm test
	./scripts/stop-test-validator.sh
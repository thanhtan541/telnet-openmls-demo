# cargo install cargo-watch
dev:
	cargo watch -x check -x test -x run

fmt:
	cargo fmt

check: fmt
	cargo check

PATTERN?="update_db"
test: check
	cargo test ${PATTERN}

test-verbose: check
	cargo test -- --nocapture

# cargo install cargo-tarpaulin
cov:
	cargo tarpaulin --ignore-tests

# rustup component add clippy
lint-check: check
	cargo clippy -- -D warnings

# rustup component add rustfmt, for CI pipeline
fmt-check:
	cargo fmt -- --check

# cargo install cargo-audit
audit:
	cargo audit

# cargo install cargo-deny
# equivalent to cargo-audit
deny-audit:
	cargo deny

build:
	cargo build

# cargo install cargo-asm
asm:
	cargo asm

# cargo install bunyan
test-log:
	export RUST_LOG="sqlx=error,info"
	export TEST_LOG=true
	cargo test ${PATTERN} | bunyan

show-todos:
	grep -rni ./src -e 'todo'
	grep -rni ./tests -e 'todo'

# cargo install cargo-udeps
scan:
	cargo +nightly udeps

docker-build:
	docker build --tag web_service --file Dockerfile .

docker-run:
	docker run -p 8000:8000 web_service

# This command is used to fix the error
# thread 'actix-rt:worker' panicked at
# 'Can not create Runtime: Os { code: 24, kind: Other, message: "Too many open files" }',
#  limit enforced by the operating system on the maximum number of open file descriptors
# (including sockets) for each process
extend-open-files:
	ulimit -n 1000

cargo-bench:
	cargo +nightly bench

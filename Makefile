# NB: the project will be built if make is invoked without any arguments.
.PHONY: default
default: build

.PHONY: build
build:
	cargo build

.PHONY: sqlite chroma ollama
run:
	./bin/run.sh

.PHONY: check
check:
	cargo check

.PHONY: chroma-clean ollama-clean sqlite-clean clean
clean:
	cargo clean

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check

.PHONY: clippy
clippy:
	cargo clippy --all-targets --all-features --tests -- -D warnings

.PHONY: sqlite
sqlite:
	./bin/sqlite.sh create && \
		./bin/sqlite.sh queries && \
			./bin/sqlite.sh migrate

.PHONY: sqlite-clean
sqlite-clean:
	./bin/sqlite.sh clean

.PHONY: ollama
ollama:
	./bin/ollama.sh run

.PHONY: ollama-clean
ollama-clean:
	./bin/ollama.sh clean

.PHONY: chroma
chroma:
	./bin/chroma.sh run

.PHONY: chroma-clean
chroma-clean:
	./bin/chroma.sh clean

.PHONY: test
test:
	cargo test --all --workspace --bins --tests --benches

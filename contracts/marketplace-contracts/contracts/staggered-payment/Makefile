default: build

all: test

test: build
	cargo test

c:
	cargo test -- --nocapture

build:
	stellar contract build
	@ls -l ../staggered-payment/target/wasm32v1-none/release/*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean

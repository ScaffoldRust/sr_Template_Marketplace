default: build

all: test

test: build
	cargo test

c:
	cargo test -- --nocapture

build:
	stellar contract build

fmt:
	cargo fmt --all

clean:
	cargo clean

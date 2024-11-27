build:
	cargo build
release:
	cargo build --release
clean:
	cargo clean
format:
	cargo fmt
coverage:
	cargo llvm-cov --html

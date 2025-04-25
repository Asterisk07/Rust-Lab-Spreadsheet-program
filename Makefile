# Makefile to compile a Rust project with Cargo and output the binary to ./target/release/spreadsheet

# Default target
all: build

# Build the project in release mode
build:
	cargo build --release
	@if [ -f ./target/release/spreadsheet ]; then \
		echo "Binary already named 'spreadsheet' in ./target/release"; \
	else \
		mv ./target/release/sheet ./target/release/spreadsheet || echo "Warning: Could not rename binary to 'spreadsheet'"; \
	fi

# Clean the target directory
clean:
	cargo clean

test: ./src Cargo.toml
	cargo test 

docs: ./src report.tex Cargo.toml
	cargo doc 
	pdflatex report.tex

coverage: src Cargo.toml
	cargo tarpaulin --release

ext1: src Cargo.toml
	cargo run 10 10 --vim

.PHONY: all build clean test docs coverage ext1

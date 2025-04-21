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

# Phony targets to avoid conflicts with files named 'all', 'build', or 'clean'
.PHONY: all build clean
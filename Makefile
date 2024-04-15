.PHONY: build check clean test lint fmt

# Set the cargo command if not set
CARGO ?= cargo

# Build the project
build:
	@echo "Building the project..."
	@$(CARGO) build --release

# Run tests
test:
	@echo "Running tests..."
	@$(CARGO) test

# Run checks: ensure no warnings, format check, and linting
check: fmt-check lint no-warnings

# Ensure no compile warnings
no-warnings:
	@echo "Checking for compile-time warnings..."
	@$(CARGO) rustc -- -D warnings

# Lint the code
lint:
	@echo "Linting the code..."
	@$(CARGO) clippy -- -D warnings

# Check code formatting
fmt-check:
	@echo "Checking code formatting..."
	@$(CARGO) fmt -- --check

# Format all code
fmt:
	@echo "Formatting all code..."
	@$(CARGO) fmt

# Clean up the project
clean:
	@echo "Cleaning up..."
	@$(CARGO) clean


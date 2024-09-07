SHELL := /bin/sh
.PHONY: help

help: ## Print a list of make options available
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' ${MAKEFILE_LIST} | sort | \
	awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

update: ## update rust and dependencies to the latest version
	@echo "Update Rust toolchains and rustup"
	rustup update
	@echo
	@echo "Update dependencies for this project"
	cp Cargo.toml Cargo.toml.bak
	head -n 6 Cargo.toml.bak > Cargo.toml
	for item in $$(awk '(NR>6 && $$0!~/features/){print $$1}' Cargo.toml.bak); do cargo add $${item}; done
	for item in $$(awk '(NR>6 && $$0~/features.*derive/){print $$1}' Cargo.toml.bak); do cargo add $${item} --features derive; done
	rm Cargo.toml.bak
	@echo 
	git diff Cargo.toml

format: ## format the project using cargo
	@rustup component add rustfmt 2> /dev/null
	cargo fmt --verbose --check

lint: ## Lint the project using cargo
	@rustup component add clippy 2> /dev/null
	cargo clippy --all -- --warn warnings

test: ## Test the project using cargo
	RUST_LOG=debug cargo test -- --nocapture

build: ## Build the project using cargo
	cargo build

clean: ## Clean the project using cargo
	cargo clean

bump: ## Bump the version number
	@echo "Current version is $(shell cargo pkgid | cut -d# -f2)"
	@read -p "Enter new version number: " version; \
	updated_version=$$(cargo pkgid | cut -d# -f2 | sed -E "s|([0-9]+\.[0-9]+\.[0-9]+)$$|$$version|"); \
	sed -i -E "s|^version = .*|version = \"$$updated_version\"|" Cargo.toml
	@echo "New version is $(shell cargo pkgid | cut -d# -f2)"
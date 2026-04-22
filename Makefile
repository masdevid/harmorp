.PHONY: setup test check fmt lint bench build clean guard

setup:
	git config core.hooksPath .githooks
	@echo "Git hooks installed from .githooks/"

check:
	cargo check

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

test:
	cargo test

test-all:
	cargo test
	cargo test --ignored -- --nocapture

bench:
	cargo bench --bench stemmer_bench

build:
	cargo build --release

guard:
	@echo "Scanning for proprietary references..."
	@if grep -rn "KBBI\|kbbi\|harmorph_stemmer" src/ tests/ benches/ examples/ README.md Cargo.toml 2>/dev/null; then \
		echo "ERROR: Proprietary references found (check for dictionary references, old package name)"; exit 1; \
	else \
		echo "OK: No proprietary references found"; \
	fi

clean:
	cargo clean

release-check: fmt lint guard test build
	@echo ""
	@echo "Release check complete. Version: $$(grep '^version' Cargo.toml | head -1 | sed 's/.*= *\"//' | sed 's/\"//')"
	@echo "Next: bump version in Cargo.toml, update CHANGELOG.md, git tag vX.Y.Z, git push --tags"

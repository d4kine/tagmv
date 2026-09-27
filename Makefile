# tagmv -- build & install helpers
#
# Override the install location with: make install BINDIR=/usr/local/bin
BINDIR ?= $(HOME)/bin
BIN    := tagmv
TARGET := target/release/$(BIN)

.PHONY: all build release test fmt fmt-check clippy check \
        install uninstall menu-install menu-uninstall menu-status clean help

all: build ## Default: debug build

build: ## Build the debug binary
	cargo build

release: ## Build the optimized release binary
	cargo build --release

test: ## Run all unit + integration tests
	cargo test

fmt: ## Format the codebase
	cargo fmt

fmt-check: ## Verify formatting without changing files
	cargo fmt --check

clippy: ## Lint with warnings treated as errors
	cargo clippy --all-targets -- -D warnings

check: fmt-check clippy test ## Run the full verification suite

install: release ## Build release and copy the binary to BINDIR
	mkdir -p "$(BINDIR)"
	cp "$(TARGET)" "$(BINDIR)/$(BIN)"
	@echo "Installed $(BIN) to $(BINDIR)/$(BIN)"
	@case ":$$PATH:" in *":$(BINDIR):"*) ;; \
		*) echo "Note: $(BINDIR) is not on your PATH.";; esac
	@echo "Context menu:"
	@"$(BINDIR)/$(BIN)" status || echo "  Run 'make menu-install' to add it."

uninstall: ## Remove the installed binary from BINDIR
	rm -f "$(BINDIR)/$(BIN)"
	@echo "Removed $(BINDIR)/$(BIN)"

menu-install: install ## Install binary + file manager context menu
	"$(BINDIR)/$(BIN)" install

menu-uninstall: ## Remove the file manager context menu
	"$(BINDIR)/$(BIN)" uninstall

menu-status: ## Show whether the context menu is installed (and enabled on macOS)
	"$(BINDIR)/$(BIN)" status

clean: ## Remove build artifacts
	cargo clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

PREFIX ?= /usr/local
BINDIR = $(DESTDIR)$(PREFIX)/bin

.PHONY: all build install uninstall clean

all: build

build:
	cd syslens-collect && cargo build --release
	cp syslens-collect/target/release/syslens-collect syslens-rust

install: build
	install -d $(BINDIR)
	install -m 755 syslens $(BINDIR)/syslens
	install -m 755 syslens-rust $(BINDIR)/syslens-rust
	@echo "Installed syslens and syslens-rust to $(BINDIR)/"

uninstall:
	rm -f $(BINDIR)/syslens $(BINDIR)/syslens-rust
	@echo "Removed syslens and syslens-rust from $(BINDIR)/"

clean:
	rm -f syslens-rust
	cd syslens-collect && cargo clean

#!/usr/bin/env bash
set -euo pipefail

REPO="kalchan12/syslens"
BRANCH="main"
BINDIR="${BINDIR:-/usr/local/bin}"
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

# ── helpers ──────────────────────────────────────────────
info()  { printf "\033[1;34m→\033[0m %s\n" "$*"; }
ok()    { printf "\033[1;32m✓\033[0m %s\n" "$*"; }
err()   { printf "\033[1;31m✗\033[0m %s\n" "$*"; exit 1; }

has_cmd() { command -v "$1" &>/dev/null; }

# ── main ─────────────────────────────────────────────────
info "Downloading syslens from ${REPO} (${BRANCH})..."

if has_cmd git && [[ -z "${NO_GIT:-}" ]]; then
  git clone --depth=1 -b "$BRANCH" "https://github.com/${REPO}.git" "$TMPDIR/syslens" 2>/dev/null ||
    err "git clone failed"
  cd "$TMPDIR/syslens"
else
  # Fallback: download tarball
  TARBALL="https://github.com/${REPO}/archive/refs/heads/${BRANCH}.tar.gz"
  if has_cmd curl; then
    curl -sfL "$TARBALL" | tar xz -C "$TMPDIR" 2>/dev/null
  elif has_cmd wget; then
    wget -qO- "$TARBALL" | tar xz -C "$TMPDIR" 2>/dev/null
  else
    err "need git, curl, or wget"
  fi
  cd "$TMPDIR/syslens-${BRANCH}"
fi

# Build Rust binary (if Rust is available)
if has_cmd cargo; then
  info "Building syslens-rust (this may take a minute)..."
  (cd syslens-collect && cargo build --release 2>/dev/null) || {
    info "Rust build failed; installing bash-only version"
  }
  if [[ -f syslens-collect/target/release/syslens-collect ]]; then
    cp syslens-collect/target/release/syslens-collect syslens-rust
    ok "syslens-rust built"
  fi
else
  info "Rust not found; installing bash-only version"
  info "  To build the Rust collector later: cd syslens-collect && cargo build --release"
fi

# Install
install -d "$BINDIR"
install -m 755 syslens "$BINDIR/syslens"
if [[ -f syslens-rust ]]; then
  install -m 755 syslens-rust "$BINDIR/syslens-rust"
  ok "Installed syslens and syslens-rust to ${BINDIR}/"
else
  ok "Installed syslens to ${BINDIR}/ (bash-only)"
fi

echo ""
info "Run: syslens        — interactive menu"
info "     syslens minimal — quick overview"
info "     syslens doctor  — health check"

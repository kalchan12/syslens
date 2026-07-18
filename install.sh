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

# ── distro detection ──────────────────────────────────────
detect_distro() {
  local id=""
  if [[ -f /etc/os-release ]]; then
    id=$(grep -oP '^ID="?\K\w+' /etc/os-release || true)
  fi
  echo "${id,,}"  # lowercase
}

# ── build dependency install ──────────────────────────────
install_build_deps() {
  local distro="$1"
  local need_sudo=false
  if [[ $EUID -ne 0 ]]; then
    if has_cmd sudo; then
      need_sudo=true
    else
      info "Not root and no sudo — skipping system package install"
      info "You may need to install: build-essential (Debian), base-devel (Arch), or gcc gcc-c++ make (Fedora)"
      return
    fi
  fi

  case "$distro" in
    debian|ubuntu|linuxmint|pop|elementary|kali|parrot|raspbian|devuan)
      info "Detected $distro — installing build-essential and curl..."
      $need_sudo && sudo apt update -qq || apt update -qq
      $need_sudo && sudo apt install -y -qq build-essential curl || apt install -y -qq build-essential curl
      ;;
    arch|manjaro|endeavouros|artix|garuda|arcolinux)
      info "Detected $distro — installing base-devel and curl..."
      $need_sudo && sudo pacman -S --noconfirm --needed base-devel curl || pacman -S --noconfirm --needed base-devel curl
      ;;
    fedora|rhel|centos|rocky|almalinux)
      info "Detected $distro — installing gcc, gcc-c++, make, and curl..."
      $need_sudo && sudo dnf install -y gcc gcc-c++ make curl || dnf install -y gcc gcc-c++ make curl
      ;;
    opensuse*|suse|sles)
      info "Detected $distro — installing gcc, gcc-c++, make, and curl..."
      $need_sudo && sudo zypper install -y gcc gcc-c++ make curl || zypper install -y gcc gcc-c++ make curl
      ;;
    alpine)
      info "Detected $distro — installing build-base and curl..."
      $need_sudo && sudo apk add --no-cache build-base curl || apk add --no-cache build-base curl
      ;;
    void)
      info "Detected $distro — installing base-devel and curl..."
      $need_sudo && sudo xbps-install -y base-devel curl || xbps-install -y base-devel curl
      ;;
    *)
      info "Unrecognized distro '$distro' — skipping system package install"
      info "You may need to install: build-essential (Debian), base-devel (Arch), or gcc gcc-c++ make (Fedora)"
      ;;
  esac
}

# ── Rust install ──────────────────────────────────────────
install_rust() {
  if has_cmd cargo; then
    return
  fi
  info "Installing Rust via rustup..."
  if ! has_cmd rustup && ! has_cmd cargo; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>/dev/null
    . "$HOME/.cargo/env"
    if ! has_cmd cargo; then
      err "rustup installed but cargo not found; try logging out and back in"
    fi
    ok "Rust installed"
  fi
}

# ── main ──────────────────────────────────────────────────
distro=$(detect_distro)
[[ -n "$distro" ]] && info "Detected distribution: $distro"

install_build_deps "$distro"
install_rust

info "Downloading syslens from ${REPO} (${BRANCH})..."

if has_cmd git && [[ -z "${NO_GIT:-}" ]]; then
  git clone --depth=1 -b "$BRANCH" "https://github.com/${REPO}.git" "$TMPDIR/syslens" 2>/dev/null ||
    err "git clone failed"
  cd "$TMPDIR/syslens"
else
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

# Build Rust binary
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

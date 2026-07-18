# syslens

**Linux System Inspection & Diagnostics Tool**

A fast, modular, cross-distribution system inspector. Uses a Rust binary (`syslens-rust`) to read `/proc`, `/sys`, and kernel interfaces directly — no external utilities required for the heavy collectors. Falls back to pure bash when the binary isn't available.

## Features

| Category | Details |
|---|---|
| **System** | Machine info, hostname, product, vendor, UUID |
| **OS** | Distribution, version, desktop environment, shell, terminal |
| **Kernel** | Release, version, modules, boot cmdline |
| **CPU** | Model, vendor, cores, frequency, load, usage, per-core freqs |
| **Memory** | Total, used, available, swap (with visual bar) |
| **Storage** | Disks, partitions, filesystem usage |
| **GPU** | Graphics devices, vendors, DRM info |
| **Battery** | Capacity, status, technology, model, watt-hours |
| **Fans** | RPM readings from sensors and hwmon |
| **Temperatures** | Thermal zones, hwmon sensors |
| **Displays** | Connected monitors, resolutions, backlight |
| **Motherboard** | Vendor, model, version, serial |
| **BIOS** | Vendor, version, date, UEFI/Legacy mode |
| **Network** | Interfaces, MAC, state, speed, traffic, gateway |
| **Audio** | Audio devices, ALSA, PulseAudio/PipeWire |
| **USB** | Device listing |
| **PCI** | Device tree with key device filtering |
| **Security** | AppArmor, SELinux, UFW, Secure Boot, ASLR, ptrace |
| **Virtualization** | Hypervisor and container detection |
| **Uptime** | Uptime display, boot time |
| **Processes** | Counts by state, top CPU consumers |

### Special Commands

- **`syslens doctor`** — Health diagnostics checking memory, CPU load, temperatures, battery, swap, disk usage, zombie processes, uptime, and ASLR
- **`syslens watch`** — Live compact monitoring with auto-refresh
- **`syslens snapshot save\|list\|diff`** — System state snapshots for tracking changes over time

### Export Formats

- `--json` — Nested JSON
- `--yaml` — YAML (requires PyYAML)
- `--markdown` — Markdown report with tables
- `--html` — Dark-themed HTML report

## Installation

### One-liner (recommended)

```bash
curl -sfL https://raw.githubusercontent.com/kalchan12/syslens/main/install.sh | bash
```

Requires `git` and `cargo` to build the Rust binary. If Rust isn't available, it installs the bash script only — fully functional but slower on some collectors.

### From source

```bash
git clone https://github.com/kalchan12/syslens.git
cd syslens
make install
```

### Manual

```bash
git clone https://github.com/kalchan12/syslens.git
cd syslens
cd syslens-collect && cargo build --release && cp target/release/syslens-collect ../syslens-rust && cd ..
sudo cp syslens syslens-rust /usr/local/bin/
```

## Usage

```
syslens                    Interactive menu
syslens minimal            Quick system overview
syslens full               All system information
syslens cpu                Show a specific section
syslens doctor             Run health diagnostics
syslens watch 5            Live monitoring (5s interval)
syslens --json             Full system info as JSON
syslens snapshot save      Take a system snapshot
syslens snapshot diff      Compare with latest snapshot
```

### Sections

`machine` `cpu` `memory` `storage` `gpu` `battery` `fans` `temps` `displays` `motherboard` `bios` `network` `audio` `usb` `pci` `os` `kernel` `security` `virt` `uptime` `processes`

## How it works

Two components working together:

- **`syslens-rust`** — reads system data directly from `/proc`, `/sys`, and kernel interfaces. Zero external crate dependencies (only `libc` for `statvfs`). Collects:
  - CPU usage (30ms sampling vs 300ms in bash)
  - Processes via `/proc/[0-9]*/stat` (no `ps` subprocess)
  - Storage via `/sys/block/*` + `statvfs` (no `lsblk`/`df`)
  - Temperature zones and hwmon sensors (no `sensors`)
  - Fan RPM from hwmon sysfs
  - USB devices from `/sys/bus/usb/devices` (no `lsusb`)
  - PCI devices with vendor/device name lookup from `pci.ids` (no `lspci`)
  - `all` subcommand runs every collector in a single invocation

- **`syslens`** — bash script handling orchestration, display rendering (Unicode box drawing, ANSI colors), interactive menus, export formats, health diagnostics, and system snapshots. When `syslens-rust` is found, it delegates collection to the binary. Otherwise, every collector falls back to pure bash reading `/proc` and `/sys` directly or (as last resort) calling external utilities.

## Requirements

- `/proc` and `/sys` (kernel interfaces, always available on Linux)
- `cargo` (only to build the Rust binary — optional)
- `python3` (only for structured export — optional, falls back to manual JSON)

The Rust binary replaces `lspci`, `lsusb`, `lsblk`, `df`, `ps`, `lscpu`, `sensors`, and `bc` — none of these are needed at runtime when `syslens-rust` is installed.

## Design

- **Independent collectors** — each system aspect collects into a flat key-value store; renderers and exporters only read from the store
- **Separate rendering** — terminal output, JSON, YAML, Markdown, and HTML are independent modules layered on the same data
- **Graceful degradation** — missing permissions, hardware, or the Rust binary never crashes; falls back through bash to direct sysfs reads to external commands
- **Cross-distribution** — no distribution-specific assumptions; avoids package-manager dependencies

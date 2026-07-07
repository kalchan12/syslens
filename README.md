# syslens

**Linux System Inspection & Diagnostics Tool**

A fast, modular, cross-distribution system inspector that gathers comprehensive hardware, operating system, and runtime information. Reads from `/proc`, `/sys`, and kernel interfaces before falling back to external utilities.

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
- **`syslens snapshot save|list|diff`** — System state snapshots for tracking changes over time

### Export Formats

- `--json` — Nested JSON
- `--yaml` — YAML (requires PyYAML)
- `--markdown` — Markdown report with tables
- `--html` — Dark-themed HTML report

## Installation

```bash
git clone https://github.com/kalchan12/syslens.git
cd syslens
chmod +x syslens
./syslens

# Optional: add to PATH
sudo ln -s "$PWD/syslens" /usr/local/bin/
```

## Usage

```
syslens                    Show all system information
syslens cpu                Show a specific section
syslens doctor             Run health diagnostics
syslens watch 5            Live monitoring (5s interval)
syslens --json             Full system info as JSON
syslens snapshot save      Take a system snapshot
syslens snapshot diff      Compare with latest snapshot
```

### Sections

`machine` `cpu` `memory` `storage` `gpu` `battery` `fans` `temps` `displays` `motherboard` `bios` `network` `audio` `usb` `pci` `os` `kernel` `security` `virt` `uptime` `processes`

## Design

syslens is built as a single bash script with:

- **Independent collectors** — each system aspect has a dedicated function storing results in a flat key-value store
- **Separate renderers** — terminal output uses Unicode box drawing and ANSI colors; export functions handle structured formats
- **Graceful degradation** — missing permissions or hardware never crashes; falls back from kernel interfaces to external commands
- **Cross-distribution** — avoids distribution-specific assumptions; tested on Debian/Ubuntu/Mint/Fedora/Arch

## Requirements

| Utility | Purpose | Fallback |
|---|---|---|
| `/proc`, `/sys` | Primary data sources | — |
| `lscpu` | CPU details | `/proc/cpuinfo` |
| `lsblk` | Block devices | — |
| `lspci` | PCI/GPU devices | — |
| `lsusb` | USB devices | — |
| `sensors` | Fans, temperatures | `/sys/class/thermal`, `/sys/class/hwmon` |
| `xrandr` | Display info | `/sys/class/drm` |
| `python3` | JSON/YAML export | Fallback to manual JSON |

Optional: `dmidecode` (extra DMI data), `PyYAML` (YAML export), `bc` (floating point math).

## License

MIT

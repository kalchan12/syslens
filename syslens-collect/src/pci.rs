use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn read_line(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn hex_u16(s: &str) -> u16 {
    u16::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0)
}

fn hex_u32(s: &str) -> u32 {
    u32::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0)
}

fn class_name(class: u32) -> String {
    let base = (class >> 16) as u8;
    let sub = ((class >> 8) & 0xff) as u8;
    let prog = (class & 0xff) as u8;

    let base_name = match base {
        0x00 => "Unclassified",
        0x01 => "Mass storage controller",
        0x02 => "Network controller",
        0x03 => "Display controller",
        0x04 => "Multimedia controller",
        0x05 => "Memory controller",
        0x06 => "Bridge",
        0x07 => "Communication controller",
        0x08 => "Generic system peripheral",
        0x09 => "Input device controller",
        0x0a => "Docking station",
        0x0b => "Processor",
        0x0c => "Serial bus controller",
        0x0d => "Wireless controller",
        0x0e => "Intelligent controller",
        0x0f => "Satellite communications controller",
        0x10 => "Encryption controller",
        0x11 => "Signal processing controller",
        0x12 => "Processing accelerators",
        0x13 => "Non-essential instrumentation",
        0x14 => "CXL",
        0x40 => "Coherent accelerator",
        0xff => "Unassigned class",
        _ => return format!("Class {:02x}", base),
    };

    let sub_name = match (base, sub) {
        (0x01, 0x00) => "SCSI storage controller",
        (0x01, 0x01) => "IDE interface",
        (0x01, 0x02) => "Floppy disk controller",
        (0x01, 0x03) => "IPI bus controller",
        (0x01, 0x04) => "RAID bus controller",
        (0x01, 0x05) => "ATA compatible controller",
        (0x01, 0x06) => "Serial ATA controller",
        (0x01, 0x07) => "Serial Attached SCSI controller",
        (0x01, 0x08) => "Non-Volatile memory controller",
        (0x02, 0x00) => "Ethernet controller",
        (0x02, 0x01) => "Token ring network controller",
        (0x02, 0x02) => "FDDI network controller",
        (0x02, 0x03) => "ATM network controller",
        (0x02, 0x04) => "ISDN controller",
        (0x02, 0x05) => "WorldFip controller",
        (0x02, 0x06) => "PICMG controller",
        (0x02, 0x07) => "Infiniband controller",
        (0x02, 0x08) => "Fabric controller",
        (0x03, 0x00) => "VGA compatible controller",
        (0x03, 0x01) => "XGA compatible controller",
        (0x03, 0x02) => "3D controller",
        (0x03, 0x03) => "Display controller",
        (0x04, 0x00) => "Video device",
        (0x04, 0x01) => "Audio device",
        (0x04, 0x02) => "Computer telephony device",
        (0x04, 0x03) => "Audio device",
        (0x06, 0x00) => "Host bridge",
        (0x06, 0x01) => "ISA bridge",
        (0x06, 0x02) => "EISA bridge",
        (0x06, 0x03) => "MicroChannel bridge",
        (0x06, 0x04) => "PCI bridge",
        (0x06, 0x05) => "PCMCIA bridge",
        (0x06, 0x06) => "NuBus bridge",
        (0x06, 0x07) => "CardBus bridge",
        (0x06, 0x08) => "RACEway bridge",
        (0x06, 0x09) => "PCI to PCI bridge",
        (0x06, 0x0a) => "InfiniBand to PCI host bridge",
        (0x07, 0x00) => "Serial controller",
        (0x07, 0x01) => "Parallel controller",
        (0x07, 0x02) => "Multiport serial controller",
        (0x07, 0x03) => "Modem",
        (0x07, 0x04) => "IEEE 1284 controller",
        (0x07, 0x05) => "Smart card",
        (0x08, 0x00) => "PIC",
        (0x08, 0x01) => "DMA controller",
        (0x08, 0x02) => "Timer",
        (0x08, 0x03) => "RTC",
        (0x08, 0x04) => "Generic PCI Hotplug controller",
        (0x08, 0x05) => "SD host controller",
        (0x08, 0x06) => "IOMMU",
        (0x08, 0x07) => "Performance counter",
        (0x0c, 0x00) => "USB controller",
        (0x0c, 0x01) => "USB controller",
        (0x0c, 0x02) => "USB controller",
        (0x0c, 0x03) => "USB controller",
        (0x0c, 0x04) => "USB controller",
        (0x0c, 0x05) => "USB controller",
        (0x0c, 0x06) => "USB controller",
        (0x0c, 0x07) => "USB controller",
        (0x0c, 0x08) => "USB controller",
        (0x0c, 0x09) => "USB controller",
        (0x0c, 0x10) => "SMBus",
        (0x0c, 0x11) => "SMBus",
        (0x0c, 0x12) => "SMBus",
        (0x11, 0x00) => "DPIO",
        (0x11, 0x01) => "Performance counters",
        (0x11, 0x02) => "Communication synchronizer",
        (0x11, 0x03) => "Signal processing management",
        (0x11, 0x04) => "Signal processing accelerator",
        _ => return base_name.to_string(),
    };

    // Check for USB controller specifics
    let usb_prog: Option<&str> = match (base, sub) {
        (0x0c, 0x03) if prog == 0x00 => Some("Universal Host Controller Interface (UHCI)"),
        (0x0c, 0x03) if prog == 0x10 => Some("Open Host Controller Interface (OHCI)"),
        (0x0c, 0x03) if prog == 0x20 => Some("Enhanced Host Controller Interface (EHCI)"),
        (0x0c, 0x03) if prog == 0x30 => Some("xHCI"),
        (0x0c, 0x03) if prog == 0x40 => Some("xHCI"),
        (0x0c, 0x03) if prog == 0x80 => Some("USB host controller"),
        (0x01, 0x01) if prog == 0x8a => Some("PIATA (parallel transport)"),
        (0x01, 0x05) if prog == 0x20 => Some("Single DMA"),
        (0x01, 0x05) if prog == 0x30 => Some("Chained DMA"),
        (0x01, 0x06) if prog == 0x01 => Some("AHCI 1.0"),
        (0x06, 0x04) if prog == 0x00 => Some("Subtractive decode"),
        (0x06, 0x04) if prog == 0x01 => Some("Half bridge"),
        (0x06, 0x04) if prog == 0x80 => Some("Normal decode"),
        _ => None,
    };

    if let Some(p) = usb_prog {
        format!("{sub_name} ({p})")
    } else {
        sub_name.to_string()
    }
}

struct PciDb {
    // vendor_id -> (vendor_name, HashMap<device_id, device_name>)
    vendors: HashMap<u16, (String, HashMap<u16, String>)>,
}

impl PciDb {
    fn load() -> Self {
        let mut db = PciDb {
            vendors: HashMap::new(),
        };

        // Try common pci.ids locations
        let paths = [
            "/usr/share/hwdata/pci.ids",
            "/usr/share/misc/pci.ids",
            "/usr/share/pci.ids",
            "/var/lib/pciutils/pci.ids",
        ];

        for path in &paths {
            if Path::new(path).exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    db.parse(&content);
                }
                break;
            }
        }

        db
    }

    fn parse(&mut self, content: &str) {
        let mut current_vendor: Option<u16> = None;

        for line in content.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if !line.starts_with('\t') {
                // Vendor line: "XXXX  Vendor Name"
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() >= 2 {
                    if let Ok(vid) = u16::from_str_radix(parts[0], 16) {
                        let name = parts[1].trim().to_string();
                        self.vendors.insert(vid, (name, HashMap::new()));
                        current_vendor = Some(vid);
                    }
                }
            } else if line.starts_with('\t') && !line.starts_with("\t\t") {
                // Device line: "\tXXXX  Device Name"
                if let Some(vid) = current_vendor {
                    let trimmed = line.trim_start_matches('\t');
                    let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
                    if parts.len() >= 2 {
                        if let Ok(did) = u16::from_str_radix(parts[0], 16) {
                            let name = parts[1].trim().to_string();
                            if let Some((_, devices)) = self.vendors.get_mut(&vid) {
                                devices.insert(did, name);
                            }
                        }
                    }
                }
            }
        }
    }

    fn lookup(&self, vendor_id: u16, device_id: u16) -> (Option<&str>, Option<&str>) {
        let vname = self
            .vendors
            .get(&vendor_id)
            .map(|(name, _)| name.as_str());
        let dname = self
            .vendors
            .get(&vendor_id)
            .and_then(|(_, devices)| devices.get(&device_id))
            .map(|s| s.as_str());
        (vname, dname)
    }
}

pub fn collect() -> Result<(), String> {
    let pci_dir = Path::new("/sys/bus/pci/devices");
    let mut idx = 0u32;

    if !pci_dir.is_dir() {
        println!("pci.count=0");
        return Ok(());
    }

    let db = PciDb::load();

    let mut devices: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(pci_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            devices.push(name);
        }
    }
    devices.sort();

    for dev_name in &devices {
        let dev_path = pci_dir.join(dev_name);

        // Format: "0000:00:01.0" -> "00:01.0" (strip domain prefix)
        let slot = dev_name
            .strip_prefix("0000:")
            .unwrap_or(dev_name);

        let vendor_str = read_line(&dev_path.join("vendor"));
        let device_str = read_line(&dev_path.join("device"));
        let class_str = read_line(&dev_path.join("class"));
        let _rev_str = read_line(&dev_path.join("revision"));

        let vendor_id = hex_u16(&vendor_str);
        let device_id = hex_u16(&device_str);
        let class_id = hex_u32(&class_str);

        let class_desc = class_name(class_id);
        let (vname, dname) = db.lookup(vendor_id, device_id);

        let description = match (vname, dname) {
            (Some(vn), Some(dn)) => {
                format!("{class_desc}: {vn} {dn}")
            }
            (Some(vn), None) => {
                format!("{class_desc}: {vn} [{vendor_id:04x}:{device_id:04x}]")
            }
            (None, _) => {
                format!("{class_desc} [{vendor_id:04x}:{device_id:04x}]")
            }
        };

        println!("pci.{idx}.slot={slot}");
        println!("pci.{idx}.description={description}");
        idx += 1;
    }

    println!("pci.count={idx}");

    Ok(())
}

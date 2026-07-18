use std::fs;
use std::path::Path;

fn read_line(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn collect() -> Result<(), String> {
    let usb_dir = Path::new("/sys/bus/usb/devices");
    let mut idx = 0u32;

    if !usb_dir.is_dir() {
        println!("usb.count=0");
        return Ok(());
    }

    let mut devices: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(usb_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();

            // Must have vendor and product IDs to be a real device
            if path.join("idVendor").exists() && path.join("idProduct").exists() {
                devices.push(name);
            }
        }
    }
    devices.sort();

    for dev_name in &devices {
        let dev_path = usb_dir.join(dev_name);

        let bus = read_line(&dev_path.join("busnum"));
        let devnum = read_line(&dev_path.join("devnum"));

        let vid = read_line(&dev_path.join("idVendor"));
        let pid = read_line(&dev_path.join("idProduct"));
        let id = format!("{vid}:{pid}");

        let product = read_line(&dev_path.join("product"));
        let manufacturer = read_line(&dev_path.join("manufacturer"));

        let description = if !manufacturer.is_empty() && !product.is_empty() {
            format!("{manufacturer} {product}")
        } else if !product.is_empty() {
            product
        } else if !manufacturer.is_empty() {
            manufacturer
        } else {
            id.clone()
        };

        println!("usb.{idx}.bus={bus}");
        println!("usb.{idx}.device={devnum}");
        println!("usb.{idx}.id={id}");
        println!("usb.{idx}.description={description}");
        idx += 1;
    }

    println!("usb.count={idx}");

    Ok(())
}

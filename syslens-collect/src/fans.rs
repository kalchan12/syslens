use std::fs;
use std::path::Path;

fn read_line(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn collect() -> Result<(), String> {
    let hwmon_dir = Path::new("/sys/class/hwmon");
    let mut fidx = 0u32;

    // Hwmon fan inputs
    if hwmon_dir.is_dir() {
        let mut hwmons: Vec<_> = fs::read_dir(hwmon_dir)
            .map_err(|e| format!("cannot read {hwmon_dir:?}: {e}"))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("hwmon"))
                    .unwrap_or(false)
            })
            .collect();
        hwmons.sort();

        for hw in hwmons {
            let hw_name = hw.file_name().and_then(|n| n.to_str()).unwrap_or("hwmon");
            let mut inputs: Vec<_> = fs::read_dir(&hw)
                .map_err(|e| format!("cannot read {:?}: {e}", hw))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    name.starts_with("fan") && name.ends_with("_input")
                })
                .collect();
            inputs.sort();

            for input_path in inputs {
                let val_str = read_line(&input_path);
                let rpm: u32 = val_str.parse().unwrap_or(0);
                if rpm == 0 {
                    continue;
                }

                let fname = input_path.file_name().and_then(|s| s.to_str()).unwrap_or("fan");
                let stem = fname.strip_suffix("_input").unwrap_or(fname);
                let key = format!("fan.hwmon.{hw_name}.{stem}");

                println!("{key}.rpm={rpm}");

                // Also store in the sequential index for rendering
                let label_path = input_path.with_file_name(format!("{stem}_label"));
                let label = if label_path.exists() {
                    read_line(&label_path)
                } else {
                    String::new()
                };
                let display_name = if label.is_empty() {
                    format!("Fan {}", fidx + 1)
                } else {
                    label
                };

                println!("fan.{fidx}.name={display_name}");
                println!("fan.{fidx}.rpm={rpm}");
                fidx += 1;
            }
        }
    }

    println!("fan.count={fidx}");

    Ok(())
}

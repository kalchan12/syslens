use std::fs;
use std::path::Path;

fn read_line(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn millideg_to_celsius(millideg: i64) -> String {
    let int_part = millideg / 1000;
    let frac_part = (millideg.abs() % 1000) / 100;
    format!("{int_part}.{frac_part}")
}

pub fn collect() -> Result<(), String> {
    let mut idx = 0u32;

    // Thermal zones
    let zone_dir = Path::new("/sys/class/thermal");
    if zone_dir.is_dir() {
        let mut zones: Vec<_> = fs::read_dir(zone_dir)
            .map_err(|e| format!("cannot read {zone_dir:?}: {e}"))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("thermal_zone"))
                    .unwrap_or(false)
            })
            .collect();
        zones.sort();

        for zone in zones {
            let temp_str = read_line(&zone.join("temp"));
            let temp_val: i64 = temp_str.parse().unwrap_or(0);
            if temp_val <= 0 {
                continue;
            }
            let zone_type = read_line(&zone.join("type"));

            println!("temp.{idx}.name={zone_type}");
            println!("temp.{idx}.celsius={}", millideg_to_celsius(temp_val));
            idx += 1;
        }
    }

    // Hwmon entries (stored separately like the bash collector does)
    let hwmon_dir = Path::new("/sys/class/hwmon");
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
                    name.starts_with("temp") && name.ends_with("_input")
                })
                .collect();
            inputs.sort();

            for input_path in inputs {
                let temp_str = read_line(&input_path);
                let temp_val: i64 = temp_str.parse().unwrap_or(0);
                if temp_val <= 0 {
                    continue;
                }

                let fname = input_path.file_name().and_then(|s| s.to_str()).unwrap_or("temp");
                let stem = fname.strip_suffix("_input").unwrap_or(fname);
                let key = format!("temp.hwmon.{hw_name}.{stem}");
                let celsius = millideg_to_celsius(temp_val);

                println!("{key}.celsius={celsius}");

                let label_path = input_path.with_file_name(format!("{stem}_label"));
                if label_path.exists() {
                    let label = read_line(&label_path);
                    if !label.is_empty() {
                        println!("{key}.label={label}");
                    }
                }
            }
        }
    }

    println!("temp.count={idx}");

    Ok(())
}

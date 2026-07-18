use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn read_line(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1099511627776 {
        let i = bytes / 1099511627776;
        let f = (bytes % 1099511627776) * 100 / 1099511627776;
        format!("{i}.{f:02} TiB")
    } else if bytes >= 1073741824 {
        let i = bytes / 1073741824;
        let f = (bytes % 1073741824) * 100 / 1073741824;
        format!("{i}.{f:02} GiB")
    } else if bytes >= 1048576 {
        let i = bytes / 1048576;
        let f = (bytes % 1048576) * 100 / 1048576;
        format!("{i}.{f:02} MiB")
    } else if bytes >= 1024 {
        let i = bytes / 1024;
        let f = (bytes % 1024) * 100 / 1024;
        format!("{i}.{f:02} KiB")
    } else {
        format!("{bytes} B")
    }
}

fn human_size_short(bytes: u64) -> String {
    let s = human_size(bytes);
    // Shorten "123.45 GiB" -> "123.5G"
    s.replace(" TiB", "T")
        .replace(" GiB", "G")
        .replace(" MiB", "M")
        .replace(" KiB", "K")
        .replace(" B", "B")
}

fn disk_sectors(path: &Path) -> u64 {
    let s = read_line(&path.join("size"));
    s.parse().unwrap_or(0)
}

fn disk_bytes(path: &Path) -> u64 {
    disk_sectors(path) * 512
}

pub fn collect() -> Result<(), String> {
    let block_dir = Path::new("/sys/block");
    let mut idx = 0u32;

    if !block_dir.is_dir() {
        println!("storage.count=0");
        println!("fs.count=0");
        return Ok(());
    }

    // Read mount points: device -> mount point
    let mut mounts: HashMap<String, String> = HashMap::new();
    if let Ok(content) = fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let dev = parts[0];
                let mnt = parts[1];
                // Extract just the basename for matching
                let dev_name = dev.trim_start_matches("/dev/");
                if !dev_name.is_empty() && dev_name != dev {
                    mounts.insert(dev_name.to_string(), mnt.to_string());
                }
            }
        }
    }

    let mut disks: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(block_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy().to_string();
            // Skip virtual devices
            if name.starts_with("loop")
                || name.starts_with("ram")
                || name.starts_with("dm-")
                || name.starts_with("zram")
                || name.starts_with("nbd")
                || name.starts_with("sr")
            {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                disks.push(name);
            }
        }
    }
    disks.sort();

    for disk_name in &disks {
        let disk_path = block_dir.join(disk_name);
        let size_bytes = disk_bytes(&disk_path);

        let human = human_size_short(size_bytes);
        let model = read_line(&disk_path.join("device").join("model"));

        println!("storage.{idx}.name={disk_name}");
        println!("storage.{idx}.size={human}");
        if !model.is_empty() {
            println!("storage.{idx}.model={model}");
        }

        // Partitions
        let mut part_idx = 0u32;
        if let Ok(entries) = fs::read_dir(&disk_path) {
            let mut parts: Vec<String> = Vec::new();
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy().to_string();
                // Partition names start with disk name + digits
                if name.starts_with(disk_name) && name.len() > disk_name.len() {
                    if let Some(ch) = name.chars().nth(disk_name.len()) {
                        if ch.is_ascii_digit() {
                            parts.push(name);
                        }
                    }
                }
            }
            parts.sort();

            for part_name in &parts {
                let part_path = disk_path.join(part_name);
                let part_bytes = disk_bytes(&part_path);
                let part_human = human_size_short(part_bytes);
                let mount = mounts.get(part_name.as_str()).cloned().unwrap_or_default();

                println!("storage.{idx}.partition.{part_idx}.name={part_name}");
                println!("storage.{idx}.partition.{part_idx}.size={part_human}");
                if !mount.is_empty() {
                    println!("storage.{idx}.partition.{part_idx}.mount={mount}");
                }
                part_idx += 1;
            }
        }

        println!("storage.{idx}.partition_count={part_idx}");
        idx += 1;
    }

    println!("storage.count={idx}");

    // Filesystem usage from statvfs
    let mut fidx = 0u32;
    // Re-read /proc/mounts for filesystem entries
    if let Ok(content) = fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }
            let device = parts[0];
            let mount_point = parts[1];
            let fstype = parts[2];

            // Skip pseudo filesystems
            if fstype == "tmpfs"
                || fstype == "devtmpfs"
                || fstype == "proc"
                || fstype == "sysfs"
                || fstype == "cgroup"
                || fstype == "cgroup2"
                || fstype == "efivarfs"
                || fstype == "devpts"
                || fstype == "pstore"
                || fstype == "securityfs"
                || fstype == "selinuxfs"
                || fstype == "hugetlbfs"
                || fstype == "mqueue"
                || fstype == "debugfs"
                || fstype == "tracefs"
                || fstype == "configfs"
                || fstype == "binfmt_misc"
                || fstype == "autofs"
                || fstype == "overlay"
            {
                continue;
            }

            let mnt_path = Path::new(mount_point);
            let stat = statvfs(mnt_path);
            if stat.total == 0 {
                continue;
            }

            let total_str = human_size_short(stat.total);
            let used_str = human_size_short(stat.total - stat.free);
            let avail_str = human_size_short(stat.avail);
            let used = stat.total - stat.free;
            let use_pct = if stat.total > 0 {
                (used * 100 / stat.total) as u32
            } else {
                0
            };

            println!("fs.{fidx}.filesystem={device}");
            println!("fs.{fidx}.size={total_str}");
            println!("fs.{fidx}.used={used_str}");
            println!("fs.{fidx}.avail={avail_str}");
            println!("fs.{fidx}.use_pct={use_pct}");
            println!("fs.{fidx}.mount={mount_point}");
            fidx += 1;
        }
    }

    println!("fs.count={fidx}");

    Ok(())
}

#[derive(Default)]
struct VfsStat {
    total: u64,
    free: u64,
    avail: u64,
}

fn statvfs(path: &Path) -> VfsStat {
    let Ok(cpath) = std::ffi::CString::new(path.to_string_lossy().as_ref()) else {
        return VfsStat::default();
    };

    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(cpath.as_ptr(), &mut stat) == 0 {
            let frsize = stat.f_frsize as u64;
            VfsStat {
                total: stat.f_blocks as u64 * frsize,
                free: (stat.f_blocks as u64 - stat.f_bfree as u64) * frsize,
                avail: stat.f_bavail as u64 * frsize,
            }
        } else {
            VfsStat::default()
        }
    }
}

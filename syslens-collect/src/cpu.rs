use std::fs;
use std::thread;
use std::time::Duration;

fn parse_stat_line(line: &str) -> Option<(u64, u64)> {
    let fields: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .take(8)
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    if fields.len() < 8 {
        return None;
    }

    let total: u64 = fields.iter().sum();
    let idle = fields[3] + fields[4]; // idle + iowait
    Some((total, idle))
}

fn read_proc_stat() -> String {
    fs::read_to_string("/proc/stat").unwrap_or_default()
}

pub fn collect() -> Result<(), String> {
    let stat1 = read_proc_stat();
    let first = stat1.lines().next().ok_or("empty /proc/stat")?;
    let (prev_total, prev_idle) = parse_stat_line(first).ok_or("failed to parse /proc/stat")?;

    thread::sleep(Duration::from_millis(30));

    let stat2 = read_proc_stat();
    let second = stat2.lines().next().ok_or("empty /proc/stat")?;
    let (total, idle) = parse_stat_line(second).ok_or("failed to parse /proc/stat")?;

    let total_delta = total.wrapping_sub(prev_total);
    let idle_delta = idle.wrapping_sub(prev_idle);

    if total_delta > 0 {
        let usage = (total_delta - idle_delta) * 100 / total_delta;
        println!("cpu.usage_percent={usage}");
    } else {
        println!("cpu.usage_percent=0");
    }

    Ok(())
}

use std::fs;
use std::path::Path;

const HERTZ: f64 = 100.0;

struct ProcInfo {
    pid: u32,
    state: String,
    utime: u64,
    stime: u64,
    starttime: u64,
    vm_rss: u64,
    cmdline: String,
}

fn read_line(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn parse_stat(path: &Path) -> Option<(u32, String, String, u64, u64, u64)> {
    let content = fs::read_to_string(path).ok()?;
    let close_paren = content.rfind(')')?;
    let open_paren = content.find('(')?;

    // pid is the part before " ("
    let pid: u32 = content[..open_paren].trim().parse().ok()?;
    let comm = content[open_paren + 1..close_paren].to_string();

    let rest = content[close_paren + 1..].trim();
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if tokens.len() < 20 {
        return None;
    }

    let state = tokens[0].to_string();
    let utime: u64 = tokens[11].parse().unwrap_or(0);
    let stime: u64 = tokens[12].parse().unwrap_or(0);
    let starttime: u64 = tokens[19].parse().unwrap_or(0);

    Some((pid, comm, state, utime, stime, starttime))
}

fn parse_status(path: &Path) -> u64 {
    let content = fs::read_to_string(path).unwrap_or_default();
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("VmRSS:") {
            // "VmRSS:    12345 kB"
            let val = val.trim();
            if let Some(kb) = val.split_whitespace().next() {
                return kb.parse().unwrap_or(0);
            }
        }
    }
    0
}

fn read_cmdline(path: &Path) -> String {
    let data = fs::read(path).unwrap_or_default();
    if data.is_empty() {
        return String::new();
    }
    // Replace null bytes with spaces, trim
    let s = String::from_utf8_lossy(&data);
    let s = s.replace('\0', " ");
    s.trim().to_string()
}

fn read_uptime() -> f64 {
    let content = read_line(Path::new("/proc/uptime"));
    content.split_whitespace().next().and_then(|s| s.parse().ok()).unwrap_or(0.0)
}

fn read_total_memory_kb() -> u64 {
    let content = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    for line in content.lines() {
        if let Some(val) = line.strip_prefix("MemTotal:") {
            let val = val.trim();
            if let Some(kb) = val.split_whitespace().next() {
                return kb.parse().unwrap_or(0);
            }
        }
    }
    0
}

pub fn collect() -> Result<(), String> {
    let proc_dir = Path::new("/proc");

    let entries: Vec<_> = fs::read_dir(proc_dir)
        .map_err(|e| format!("cannot read /proc: {e}"))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            !name.is_empty() && name.bytes().all(|b| b.is_ascii_digit())
        })
        .collect();

    let uptime = read_uptime();
    let total_mem_kb = read_total_memory_kb();
    let mut processes: Vec<ProcInfo> = Vec::new();

    for proc_path in entries {
        let pid_str = proc_path.file_name().and_then(|n| n.to_str()).unwrap_or("0");
        let pid: u32 = pid_str.parse().unwrap_or(0);
        if pid == 0 {
            continue;
        }

        let stat_path = proc_path.join("stat");
        let Some((_, comm, state, utime, stime, starttime)) = parse_stat(&stat_path) else {
            continue;
        };

        let status_path = proc_path.join("status");
        let vm_rss = parse_status(&status_path);

        let cmdline_path = proc_path.join("cmdline");
        let cmdline = read_cmdline(&cmdline_path);
        let display_cmd = if cmdline.is_empty() {
            format!("[{comm}]")
        } else {
            cmdline.chars().take(55).collect()
        };

        processes.push(ProcInfo {
            pid,
            state,
            utime,
            stime,
            starttime,
            vm_rss,
            cmdline: display_cmd,
        });
    }

    // Count states
    let total = processes.len();
    let mut running = 0u32;
    let mut sleeping = 0u32;
    let mut zombie = 0u32;
    let mut stopped = 0u32;

    for p in &processes {
        match p.state.as_str() {
            "R" => running += 1,
            "S" | "D" => sleeping += 1,
            "Z" => zombie += 1,
            "T" | "t" => stopped += 1,
            _ => {}
        }
    }

    println!("process.total={total}");
    println!("process.running={running}");
    println!("process.sleeping={sleeping}");
    println!("process.zombie={zombie}");
    println!("process.stopped={stopped}");

    // Top processes by approximate CPU usage
    // CPU% = (total_time / HERTZ) / (uptime - starttime/HERTZ) * 100
    // MEM% = vm_rss / total_mem_kb * 100
    let mut with_cpu: Vec<(f64, f64, &ProcInfo)> = processes
        .iter()
        .map(|p| {
            let total_time = (p.utime + p.stime) as f64 / HERTZ;
            let runtime = uptime - p.starttime as f64 / HERTZ;
            let cpu_pct = if runtime > 0.0 {
                (total_time / runtime) * 100.0
            } else {
                0.0
            };
            let mem_pct = if total_mem_kb > 0 {
                (p.vm_rss as f64 / total_mem_kb as f64) * 100.0
            } else {
                0.0
            };
            (cpu_pct, mem_pct, p)
        })
        .collect();

    // Sort by CPU descending, take top 10
    with_cpu.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let top_count = with_cpu.len().min(10);
    println!("process.top_count={top_count}");
    for (i, &(cpu_pct, mem_pct, p)) in with_cpu.iter().enumerate().take(top_count) {
        println!("process.top.{i}.pid={}", p.pid);
        println!("process.top.{i}.cpu={cpu_pct:.1}");
        println!("process.top.{i}.mem={mem_pct:.1}");
        println!("process.top.{i}.cmd={}", p.cmdline);
    }

    // Top by RSS
    let mut by_rss: Vec<&ProcInfo> = processes.iter().filter(|p| p.vm_rss > 0).collect();
    by_rss.sort_by(|a, b| b.vm_rss.cmp(&a.vm_rss));
    let top_rss_count = by_rss.len().min(10);
    println!("process.topmem_count={top_rss_count}");
    for (i, p) in by_rss.iter().enumerate().take(top_rss_count) {
        println!("process.topmem.{i}.pid={}", p.pid);
        println!("process.topmem.{i}.mem={}", p.vm_rss);
        println!("process.topmem.{i}.cmd={}", p.cmdline);
    }

    Ok(())
}

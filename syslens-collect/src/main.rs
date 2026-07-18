mod cpu;
mod fans;
mod pci;
mod process;
mod storage;
mod temps;
mod usb;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    let result = match cmd {
        "cpu-usage" => cpu::collect(),
        "temps" => temps::collect(),
        "processes" => process::collect(),
        "storage" => storage::collect(),
        "fans" => fans::collect(),
        "usb" => usb::collect(),
        "pci" => pci::collect(),
        "help" => {
            eprintln!("Usage: syslens-collect <command>");
            eprintln!("Commands:");
            eprintln!("  cpu-usage    CPU usage percentage");
            eprintln!("  temps        Temperature sensors");
            eprintln!("  processes    Process counts and top consumers");
            eprintln!("  storage      Disk and filesystem information");
            eprintln!("  fans         Fan speeds");
            eprintln!("  usb          USB devices");
            eprintln!("  pci          PCI devices");
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {cmd}");
            eprintln!("Try: syslens-collect help");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

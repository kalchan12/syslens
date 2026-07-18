mod cpu;
mod process;
mod temps;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    let result = match cmd {
        "cpu-usage" => cpu::collect(),
        "temps" => temps::collect(),
        "processes" => process::collect(),
        "help" => {
            eprintln!("Usage: syslens-collect <command>");
            eprintln!("Commands:");
            eprintln!("  cpu-usage    CPU usage percentage");
            eprintln!("  temps        Temperature sensors");
            eprintln!("  processes    Process counts and top consumers");
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

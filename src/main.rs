mod parser;
mod types;

use anyhow::Result;
use clap::Parser;
//use types::RegionType;

#[derive(Parser)]
#[command(name = "memtrace", about = "Inspect memory layout of a running process")]
struct Args {
    /// PID of the process to inspect
    pid: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let regions = parser::parse_maps(args.pid)?;

    println!("{:<20} {:>10}  {:<6}  {:<}", "ADDRESS RANGE", "SIZE", "PERMS", "LABEL");
    println!("{}", "-".repeat(60));

    for r in &regions {
        let perms = format!(
            "{}{}{}{}",
            if r.perms.read    { "r" } else { "-" },
            if r.perms.write   { "w" } else { "-" },
            if r.perms.execute { "x" } else { "-" },
            if r.perms.shared  { "s" } else { "p" },
        );

        let suspicious = if r.perms.is_suspicious() { " ⚠ RWX" } else { "" };
        let size_kb = r.size / 1024;

        println!(
            "{:#018x}  {:>8} KB  {:<6}  {}{}",
            r.start, size_kb, perms, r.label, suspicious
        );
    }

    println!("\nTotal regions: {}", regions.len());
    println!(
        "Suspicious (rwx): {}",
        regions.iter().filter(|r| r.perms.is_suspicious()).count()
    );

    Ok(())
}
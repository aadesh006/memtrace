mod parser;
mod types;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "memtrace", about = "Inspect memory layout of a running process")]
struct Args {
    pid: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let regions = parser::parse_maps(args.pid)?;

    println!(
        "{:<16}  {:>10}  {:>8}  {:>8}  {:>13}  {:<6}  {}",
        "ADDRESS", "VIRT", "RSS", "PSS", "PRIVATE_DIRTY", "PERMS", "LABEL"
    );
    println!("{}", "-".repeat(85));

    for r in &regions {
        let perms = format!(
            "{}{}{}{}",
            if r.perms.read    { "r" } else { "-" },
            if r.perms.write   { "w" } else { "-" },
            if r.perms.execute { "x" } else { "-" },
            if r.perms.shared  { "s" } else { "p" },
        );
        let suspicious = if r.perms.is_suspicious() { " ⚠ RWX" } else { "" };

        println!(
            "{:#016x}  {:>8} KB  {:>6} KB  {:>6} KB  {:>11} KB  {:<6}  {}{}",
            r.start,
            r.size / 1024,
            r.rss,
            r.pss,
            r.private_dirty,
            perms,
            r.label,
            suspicious
        );
    }

    // Summary stats
    let total_rss: u64         = regions.iter().map(|r| r.rss).sum();
    let total_private: u64     = regions.iter().map(|r| r.private_dirty).sum();
    let suspicious_count       = regions.iter().filter(|r| r.perms.is_suspicious()).count();

    println!("{}", "-".repeat(85));
    println!("Total regions:       {}", regions.len());
    println!("Total RSS:           {} KB", total_rss);
    println!("Total Private Dirty: {} KB", total_private);
    println!("Suspicious (rwx):    {}", suspicious_count);

    Ok(())
}
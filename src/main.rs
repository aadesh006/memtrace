mod parser;
mod types;

use anyhow::Result;
use clap::Parser;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "memtrace", about = "Inspect memory layout of a running process")]
struct Args {
    /// PID of the process to inspect
    pid: u32,

    /// Watch mode — refresh every N seconds
    #[arg(short, long)]
    watch: bool,

    /// Refresh interval in seconds (default: 2)
    #[arg(short, long, default_value_t = 2)]
    interval: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.watch {
        run_watch_mode(args.pid, args.interval)?;
    } else {
        print_snapshot(args.pid)?;
    }

    Ok(())
}

fn run_watch_mode(pid: u32, interval: u64) -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);

    // Register Ctrl+C handler
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set Ctrl+C handler");

    println!("Watching PID {} — refreshing every {}s. Press Ctrl+C to stop.\n", pid, interval);

    while running.load(Ordering::SeqCst) {
        // Clear the terminal screen
        print!("\x1B[2J\x1B[H");

        match print_snapshot(pid) {
            Ok(_)  => {}
            Err(e) => {
                // Process likely exited
                println!("Process ended or unreadable: {}", e);
                break;
            }
        }


        let chunks = interval * 10;
        for _ in 0..chunks {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    println!("\nStopped.");
    Ok(())
}

fn print_snapshot(pid: u32) -> Result<()> {
    let regions = parser::parse_maps(pid)?;

    println!(
        "{:<16}  {:>8}  {:>8}  {:>8}  {:>13}  {:<6}  {}",
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
            "{:#016x}  {:>6} KB  {:>6} KB  {:>6} KB  {:>11} KB  {:<6}  {}{}",
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

    let total_rss: u64     = regions.iter().map(|r| r.rss).sum();
    let total_private: u64 = regions.iter().map(|r| r.private_dirty).sum();
    let suspicious_count   = regions.iter().filter(|r| r.perms.is_suspicious()).count();

    println!("{}", "-".repeat(85));
    println!("Total regions:        {}", regions.len());
    println!("Total RSS:            {} KB", total_rss);
    println!("Total Private Dirty:  {} KB", total_private);
    println!("Suspicious (rwx):     {}", suspicious_count);

    Ok(())
}
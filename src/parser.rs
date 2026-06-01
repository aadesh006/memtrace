use crate::types::{MemoryRegion, Permissions, RegionType};
use anyhow::{Context, Result};
use std::fs;

pub fn parse_maps(pid: u32) -> Result<Vec<MemoryRegion>> {
    let path = format!("/proc/{}/smaps", pid);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path))?;

    let regions = split_into_blocks(&content)
        .iter()
        .filter_map(|block| parse_block(block))
        .collect();

    Ok(regions)
}

fn split_into_blocks<'a>(content: &'a str) -> Vec<Vec<&'a str>> {
    let mut blocks: Vec<Vec<&str>> = Vec::new();
    let mut current_block: Vec<&str> = Vec::new();

    for line in content.lines() {
        let is_header = line
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_hexdigit());

        if is_header && !current_block.is_empty() {
            blocks.push(current_block);
            current_block = Vec::new();
        }

        current_block.push(line);
    }


    if !current_block.is_empty() {
        blocks.push(current_block);
    }

    blocks
}

fn parse_block(block: &[&str]) -> Option<MemoryRegion> {
    let header = block.first()?;
    let (start, end, perms, label) = parse_header(header)?;


    let mut rss = 0u64;
    let mut pss = 0u64;
    let mut private_dirty = 0u64;

    for line in &block[1..] {
        if let Some((key, val)) = line.split_once(':') {
            let kb = parse_kb(val);
            match key.trim() {
                "Rss"           => rss = kb,
                "Pss"           => pss = kb,
                "Private_Dirty" => private_dirty = kb,
                _               => {}
            }
        }
    }

    Some(MemoryRegion {
        start,
        end,
        size: end - start,
        perms,
        region_type: classify_region(&label),
        label,
        rss,
        pss,
        private_dirty,
    })
}

// Parses the header line of a block
// "7f3a2b000000-7f3a2b021000 r--p 00000000 08:01 123 /usr/lib/libc.so.6"
fn parse_header(line: &str) -> Option<(u64, u64, Permissions, String)> {
    let mut parts = line.split_whitespace();

    let range = parts.next()?;
    let (start_str, end_str) = range.split_once('-')?;
    let start = u64::from_str_radix(start_str, 16).ok()?;
    let end   = u64::from_str_radix(end_str,   16).ok()?;

    let perms_str = parts.next()?;
    let perms = parse_permissions(perms_str);

    // skip offset, dev, inode
    parts.next();
    parts.next();
    parts.next();

    let label = parts.next().unwrap_or("").to_string();

    Some((start, end, perms, label))
}

// Parses "  192 kB" → 192
fn parse_kb(s: &str) -> u64 {
    s.split_whitespace()
     .next()
     .and_then(|n| n.parse().ok())
     .unwrap_or(0)
}

fn parse_permissions(s: &str) -> Permissions {
    let chars: Vec<char> = s.chars().collect();
    Permissions {
        read:    chars.get(0).map_or(false, |&c| c == 'r'),
        write:   chars.get(1).map_or(false, |&c| c == 'w'),
        execute: chars.get(2).map_or(false, |&c| c == 'x'),
        shared:  chars.get(3).map_or(false, |&c| c == 's'),
    }
}

fn classify_region(label: &str) -> RegionType {
    match label {
        "[heap]"                                         => RegionType::Heap,
        "[stack]"                                        => RegionType::Stack,
        ""                                               => RegionType::Anonymous,
        l if l.ends_with(".so") || l.contains(".so.")   => RegionType::SharedLib,
        _                                                => RegionType::Other,
    }
}
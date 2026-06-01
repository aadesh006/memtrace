use crate::types::{MemoryRegion, Permissions, RegionType};
use anyhow::{Context, Result};
use std::fs;

pub fn parse_maps(pid: u32) -> Result<Vec<MemoryRegion>> {
    let path = format!("/proc/{}/maps", pid);
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path))?;

    let regions = content
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| parse_line(line))
        .collect();

    Ok(regions)
}

fn parse_line(line: &str) -> Option<MemoryRegion> {
    let mut parts = line.split_whitespace();

    // Parse address range: "7f3a2b000000-7f3a2b021000"
    let range = parts.next()?;
    let (start_str, end_str) = range.split_once('-')?;
    let start = u64::from_str_radix(start_str, 16).ok()?;
    let end = u64::from_str_radix(end_str, 16).ok()?;

    // Parse permissions: "r-xp"
    let perms_str = parts.next()?;
    let perms = parse_permissions(perms_str);

    // Skip offset, dev, inode
    parts.next(); // offset
    parts.next(); // dev
    parts.next(); // inode

    // Label is the rest (pathname or empty)
    let label = parts.next().unwrap_or("").to_string();
    let region_type = classify_region(&label);

    Some(MemoryRegion {
        start,
        end,
        size: end - start,
        perms,
        region_type,
        label,
    })
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
        "[heap]"            => RegionType::Heap,
        "[stack]"           => RegionType::Stack,
        ""                  => RegionType::Anonymous,
        l if l.ends_with(".so") || l.contains(".so.") => RegionType::SharedLib,
        _                   => RegionType::Other,
    }
}
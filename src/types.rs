#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub perms: Permissions,
    pub region_type: RegionType,
    pub label: String,
    //actual physical memory usage
    pub rss: u64,
    pub pss: u64,
    pub private_dirty: u64,
}

#[derive(Debug, Clone)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub shared: bool,
}

impl Permissions {
    // rwx regions are a security red flag
    pub fn is_suspicious(&self) -> bool {
        self.read && self.write && self.execute
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegionType {
    Heap,
    Stack,
    SharedLib,
    Executable,
    Anonymous,
    Other,
}
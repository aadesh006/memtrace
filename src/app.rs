use crate::types::MemoryRegion;

pub struct App {
    pub pid: u32,
    pub regions: Vec<MemoryRegion>,
    pub scroll: usize,
    pub selected: usize,
    pub visible_rows: usize,
    pub should_quit: bool,
    pub error: Option<String>,
}

impl App {
    pub fn new(pid: u32) -> Self {
        App {
            pid,
            regions: Vec::new(),
            scroll: 0,
            selected: 0,
            visible_rows: 20,
            should_quit: false,
            error: None,
        }
    }

    pub fn refresh(&mut self) {
        match crate::parser::parse_maps(self.pid) {
            Ok(regions) => {
                self.regions = regions;
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Failed to read process {}: {}", self.pid, e));
            }
        }
    }

    pub fn scroll_down(&mut self) {
        let max = self.regions.len().saturating_sub(1);
        if self.selected < max {
            self.selected += 1;
            if self.selected >= self.scroll + self.visible_rows {
                self.scroll += 1;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll {
                self.scroll -= 1;
            }
        }
    }

    pub fn total_rss(&self) -> u64 {
        self.regions.iter().map(|r| r.rss).sum()
    }

    pub fn total_private(&self) -> u64 {
        self.regions.iter().map(|r| r.private_dirty).sum()
    }

    pub fn suspicious_count(&self) -> usize {
        self.regions.iter().filter(|r| r.perms.is_suspicious()).count()
    }
}
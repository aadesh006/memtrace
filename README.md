# memtrace

A CLI tool for inspecting the memory layout of any running Linux process. Reads directly from `/proc/<pid>/smaps` — no instrumentation, no recompilation, no root required.

```
ADDRESS            VIRT        RSS       PSS   PRIVATE_DIRTY  PERMS  LABEL
---------------------------------------------------------------------------
0x00576a1b691000   192 KB   192 KB    96 KB          0 KB   r--p   /usr/bin/bash
0x00576a1b6c1000   956 KB   956 KB   488 KB          0 KB   r-xp   /usr/bin/bash
0x00576a46d75000  1716 KB  1608 KB  1608 KB       1608 KB   rw-p   [heap]
0x007ffdcc706000   132 KB   108 KB   108 KB        108 KB   rw-p   [stack]
...
---------------------------------------------------------------------------
Total regions:        41
Total RSS:          5552 KB
Total Private Dirty: 1900 KB
Suspicious (rwx):      0
```

---

## What it shows

| Column | What it means |
|---|---|
| `VIRT` | Virtual address space reserved (not actual RAM) |
| `RSS` | Physical RAM currently resident |
| `PSS` | Proportional share — shared pages divided fairly across processes |
| `PRIVATE_DIRTY` | Pages only this process owns and has modified — the truest cost |
| `PERMS` | Read/write/execute/shared flags |

`⚠ RWX` flags any region marked readable + writable + executable — a classic indicator of JIT compilation or suspicious code injection.

---

## Why PSS and Private_Dirty matter

Tools like `top` and `htop` show RSS. That's misleading. If bash loads `libc.so.6` and 30 other processes share the same physical pages, RSS attributes the full cost to every process. PSS divides it proportionally. Private_Dirty shows what only *you* own.

Example from a real bash process:
```
libc.so.6  r-xp   RSS: 1440 KB   PSS: 46 KB
```
1440 KB is resident. But since ~30 processes share it, bash is only responsible for 46 KB.

---

## Installation

**Prerequisites:** Rust toolchain (`rustup`), Linux (or WSL).

```bash
git clone https://github.com/aadesh006/memtrace
cd memtrace
cargo build --release
```

The binary is at `target/release/memtrace`. Optionally install it globally:

```bash
cargo install --path .
```

---

## Usage

```bash
# Inspect any process by PID
memtrace <pid>

# Inspect your current shell
memtrace $$

# Find a PID first
ps aux | grep firefox
memtrace 4821
```

---

## How it works

Linux exposes live process information through the `/proc` virtual filesystem. No special kernel modules or elevated permissions are needed — the kernel makes this data available to any user for processes they own.

`memtrace` reads `/proc/<pid>/smaps`, which contains one block per memory region:

```
576a1b691000-576a1b6c1000 r--p 00000000 08:20 123  /usr/bin/bash
Size:                192 kB
Rss:                 192 kB
Pss:                  96 kB
Private_Dirty:         0 kB
...
```

It parses these blocks into structured `MemoryRegion` types, then renders them as a formatted table with summary statistics.

---

## Project structure

```
src/
├── main.rs       — CLI entry point, output formatting
├── parser.rs     — /proc/<pid>/smaps parsing logic
└── types.rs      — MemoryRegion struct, Permissions, RegionType
```

---

## Limitations

- **Linux only** — depends on `/proc`, not available on macOS or Windows (WSL works)
- **Own processes only** — reading `/proc/<pid>/smaps` for another user's process requires root
- **Static snapshot** — no live refresh yet (planned)

---

## Roadmap

- [ ] Live polling mode — refresh every N seconds, watch heap grow in real time
- [ ] `ratatui` TUI — interactive scrollable interface with color-coded region types
- [ ] Filter flags — `--heap`, `--anon`, `--rwx` to focus on specific region types
- [ ] Process name lookup — `memtrace firefox` instead of needing the PID

---

## Built with

- [`clap`](https://github.com/clap-rs/clap) — CLI argument parsing
- [`anyhow`](https://github.com/dtolnay/anyhow) — ergonomic error handling
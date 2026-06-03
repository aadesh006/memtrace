mod app;
mod parser;
mod types;
mod ui;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "memtrace", about = "Inspect memory layout of a running process")]
struct Args {
    /// PID of the process to inspect
    pid: u32,

    /// Refresh interval in seconds (default: 2)
    #[arg(short, long, default_value_t = 2)]
    interval: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, args.pid, args.interval);

    // Always restore terminal even if we crashed
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    pid: u32,
    interval: u64,
) -> Result<()> {
    let mut app = App::new(pid);
    app.refresh(); // initial load

    let refresh_interval = Duration::from_secs(interval);
    let mut last_refresh = Instant::now();


    loop {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                // KeyEventKind::Press check prevents double-firing on Windows/WSL
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                        KeyCode::Up   | KeyCode::Char('k') => app.scroll_up(),
                        KeyCode::Char('r') => {
                            app.refresh();
                            last_refresh = Instant::now();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }


        if last_refresh.elapsed() >= refresh_interval {
            app.refresh();
            last_refresh = Instant::now();
        }
    }

    Ok(())
}
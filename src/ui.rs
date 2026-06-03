use crate::app::App;
use crate::types::RegionType;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(frame.area());

    // Update how many rows are visible based on actual terminal height
    app.visible_rows = chunks[1].height.saturating_sub(3) as usize;

    draw_header(frame, app, chunks[0]);
    draw_table(frame, app, chunks[1]);
    draw_footer(frame, app, chunks[2]);
}

fn draw_header(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = format!(" memtrace — PID {} ", app.pid);
    let subtitle = " ↑↓ scroll   q quit   r refresh ";

    let header = Paragraph::new(Line::from(vec![
        Span::styled(&title, Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)),
        Span::styled(subtitle, Style::default()
            .fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn draw_table(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let header_cells = ["ADDRESS", "VIRT", "RSS", "PSS", "PRIV_DIRTY", "PERMS", "LABEL"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)));

    let header_row = Row::new(header_cells).height(1).bottom_margin(1);

    // Only render the visible slice of regions
    let visible = &app.regions[app.scroll..app.regions.len().min(app.scroll + app.visible_rows + 20)];

    let rows = visible.iter().enumerate().map(|(i, r)| {
        let actual_index = app.scroll + i;
        let is_selected = actual_index == app.selected;

        let perms = format!(
            "{}{}{}{}",
            if r.perms.read    { "r" } else { "-" },
            if r.perms.write   { "w" } else { "-" },
            if r.perms.execute { "x" } else { "-" },
            if r.perms.shared  { "s" } else { "p" },
        );

        // Color each row based on region type
        let region_color = match r.region_type {
            RegionType::Heap       => Color::Green,
            RegionType::Stack      => Color::Blue,
            RegionType::SharedLib  => Color::Magenta,
            RegionType::Anonymous  => Color::DarkGray,
            RegionType::Executable => Color::Cyan,
            RegionType::Other      => Color::White,
        };

        let label = if r.perms.is_suspicious() {
            format!("{} ⚠ RWX", r.label)
        } else {
            r.label.clone()
        };

        let cells = vec![
            Cell::from(format!("{:#016x}", r.start)),
            Cell::from(format!("{} KB", r.size / 1024)),
            Cell::from(format!("{} KB", r.rss)),
            Cell::from(format!("{} KB", r.pss)),
            Cell::from(format!("{} KB", r.private_dirty)),
            Cell::from(perms),
            Cell::from(label),
        ];

        let row_style = if is_selected {
            Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(region_color)
        };

        Row::new(cells).style(row_style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(18), // address
            Constraint::Length(9),  // virt
            Constraint::Length(9),  // rss
            Constraint::Length(9),  // pss
            Constraint::Length(11), // priv_dirty
            Constraint::Length(6),  // perms
            Constraint::Min(20),    // label
        ],
    )
    .header(header_row)
    .block(Block::default().borders(Borders::ALL).title(" Regions "));

    frame.render_widget(table, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let suspicious = app.suspicious_count();
    let suspicious_style = if suspicious > 0 {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let text = vec![
        Line::from(vec![
            Span::raw(" Regions: "),
            Span::styled(
                format!("{}", app.regions.len()),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw("   Total RSS: "),
            Span::styled(
                format!("{} KB", app.total_rss()),
                Style::default().fg(Color::Green),
            ),
            Span::raw("   Private Dirty: "),
            Span::styled(
                format!("{} KB", app.total_private()),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("   Suspicious (rwx): "),
            Span::styled(format!("{}", suspicious), suspicious_style),
        ]),
        Line::from(vec![
            Span::styled(" Green ", Style::default().fg(Color::Green)),
            Span::raw("heap  "),
            Span::styled(" Blue ", Style::default().fg(Color::Blue)),
            Span::raw("stack  "),
            Span::styled(" Magenta ", Style::default().fg(Color::Magenta)),
            Span::raw("shared lib  "),
            Span::styled(" Gray ", Style::default().fg(Color::DarkGray)),
            Span::raw("anonymous"),
        ]),
    ];

    let footer = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Summary "));

    frame.render_widget(footer, area);
}
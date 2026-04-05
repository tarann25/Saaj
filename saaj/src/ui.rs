// === src/ui.rs ===
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, AppMode, RepeatMode};
use crate::theme::Theme;
use chrono::Local;

pub fn draw(frame: &mut Frame, app: &mut App) {
    if app.show_splash {
        draw_splash(frame, app, frame.area());
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(0),    // Main content area
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    draw_title_bar(frame, app, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(38), // Library panel
            Constraint::Percentage(62), // Right column
        ])
        .split(chunks[1]);

    draw_library(frame, app, main_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Download prompt
            Constraint::Min(0),         // Now Playing
            Constraint::Percentage(45), // Bottom row
        ])
        .split(main_chunks[1]);

    draw_download(frame, app, right_chunks[0]);
    draw_now_playing(frame, app, right_chunks[1]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Spectrum panel
            Constraint::Percentage(40), // Controls panel
        ])
        .split(right_chunks[2]);

    draw_spectrum(frame, app, bottom_chunks[0]);
    draw_controls(frame, app, bottom_chunks[1]);

    draw_status_bar(frame, app, chunks[2]);
}

fn draw_splash(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Block::default().style(Style::default().bg(Theme::BG)), area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),  // top padding
            Constraint::Length(10),      // title block (6 lines + margin)
            Constraint::Length(2),       // subtitle
            Constraint::Percentage(40),  // bottom padding
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),  // centered content column
            Constraint::Percentage(20),
        ])
        .split(vertical[1]);

    let title_art = vec![
        "███████╗ █████╗  █████╗      ██╗",
        "██╔════╝██╔══██╗██╔══██╗     ██║",
        "███████╗███████║███████║     ██║",
        "╚════██║██╔══██║██╔══██║██   ██║",
        "███████║██║  ██║██║  ██║╚█████╔╝",
        "╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚════╝",
    ];

    let mut lines = vec![];
    for (i, line) in title_art.into_iter().enumerate() {
        let style = if app.splash_tick % 6 == i as u64 {
            Style::default().fg(Theme::TEXT)
        } else {
            Style::default().fg(Theme::ACCENT_BRIGHT)
        };
        lines.push(Line::from(Span::styled(line, style)));
    }

    let title_p = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(title_p, horizontal[1]);

    let subtitle_horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),  // centered content column
            Constraint::Percentage(20),
        ])
        .split(vertical[2]);

    let subtitle_text = if app.splash_tick % 8 < 4 {
        "a music player  ·  press enter to continue"
    } else {
        "a music player  ·  "
    };

    let subtitle_p = Paragraph::new(Line::from(Span::styled(subtitle_text, Style::default().fg(Theme::MUTED)))).alignment(Alignment::Center);
    frame.render_widget(subtitle_p, subtitle_horizontal[1]);
}

fn styled_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(Theme::ACCENT_BRIGHT),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Theme::BORDER))
}

fn draw_title_bar(frame: &mut Frame, _app: &App, area: Rect) {
    let time_str = Local::now().format("%H:%M").to_string();
    let left_span = Span::styled("♪ SAAJ", Style::default().fg(Theme::ACCENT_BRIGHT).add_modifier(Modifier::BOLD));
    let right_span = Span::styled(&time_str, Style::default().fg(Theme::MUTED));
    
    let total_width = area.width as usize;
    let left_len = left_span.content.chars().count();
    let right_len = right_span.content.chars().count();
    let dashes_len = total_width.saturating_sub(left_len + right_len);
    let dashes = "─".repeat(dashes_len);
    
    let mid_span = Span::styled(dashes, Style::default().fg(Theme::BORDER_DIM));

    let p = Paragraph::new(Line::from(vec![left_span, mid_span, right_span]));
    frame.render_widget(p, area);
}

fn draw_download(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block(if app.mode == AppMode::DownloadPrompt { "download [EDITING]" } else { "download (press 'd')" });
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = match app.mode {
        AppMode::Downloading => Span::styled("Downloading via yt-dlp... Please wait.", Style::default().fg(Theme::AMBER)),
        AppMode::DownloadPrompt => Span::styled(format!("> {}█", app.download_input), Style::default().fg(Theme::TEXT)),
        _ => Span::styled("> Type a song name to download...", Style::default().fg(Theme::DIM)),
    };
    
    frame.render_widget(Paragraph::new(Line::from(text)), inner);
}

fn draw_library(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = styled_block("library");
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let mut items = vec![];

    for (i, track) in app.tracks.iter().enumerate() {
        let duration_mins = track.duration_secs / 60;
        let duration_secs = track.duration_secs % 60;
        let duration_str = format!("{}:{:02}", duration_mins, duration_secs);
        
        let prefix = if i == app.selected { "▶ " } else { "  " };
        
        let title_w = inner_area.width.saturating_sub(15) as usize;
        let title_truncated = if track.title.chars().count() > title_w {
            let mut s = track.title.chars().take(title_w.saturating_sub(1)).collect::<String>();
            s.push('…');
            s
        } else {
            format!("{:<width$}", track.title, width = title_w)
        };

        let style = if i == app.selected {
            Style::default().bg(Theme::SURFACE2).fg(Theme::TEXT).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Line::from(vec![
            Span::styled(prefix, Style::default().fg(if i == app.selected { Theme::TEXT } else { Theme::DIM })),
            Span::styled(format!("{:02}  ", i + 1), Style::default().fg(Theme::DIM)),
            Span::styled(format!("{}  ", title_truncated), Style::default().fg(if i == app.selected { Theme::TEXT } else { Theme::MUTED })),
            Span::styled(duration_str, Style::default().fg(Theme::DIM)),
        ]);

        items.push(ListItem::new(line).style(style));
    }

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(app.selected));

    let list = List::new(items);
    frame.render_stateful_widget(list, inner_area, &mut list_state);
}

fn draw_now_playing(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = styled_block("now playing");
    let block_inner_area = block.inner(area);
    frame.render_widget(block, area);

    if app.tracks.is_empty() { return; }

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18), // album art (16 cols + 2 padding)
            Constraint::Min(0),     // track info and controls
        ])
        .split(block_inner_area);

    let track = app.current_track().clone();

    let mut lines = vec![];
    if let Some(ref mut protocol) = app.current_album_art {
        let image = ratatui_image::StatefulImage::default();
        frame.render_stateful_widget(image, inner[0], protocol);
    } else {
        let art_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Theme::BORDER_DIM));
        frame.render_widget(art_block, inner[0]);
    }
    lines.push(Line::from(Span::styled(&track.title, Style::default().fg(Theme::TEXT).add_modifier(Modifier::BOLD))));
    lines.push(Line::from(Span::styled(format!("{} · {} · {}", track.artist, track.album, track.year), Style::default().fg(Theme::MUTED))));
    lines.push(Line::from(vec![
        Span::styled(format!("[{}]", track.format), Style::default().fg(Theme::ACCENT_BRIGHT)),
        Span::raw(" "),
        Span::styled(format!("[{}]", track.bitrate), Style::default().fg(Theme::MUTED)),
    ]));
    
    lines.push(Line::from(Span::styled("─".repeat(inner[1].width as usize), Style::default().fg(Theme::BORDER_DIM))));
    
    let p = Paragraph::new(lines).alignment(Alignment::Center);
    
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(1), // progress text
            Constraint::Length(1), // gauge
            Constraint::Min(0),    // empty space
            Constraint::Length(1), // controls
        ])
        .split(inner[1]);
        
    frame.render_widget(p, layout[0]);
    
    let progress_percent = app.progress_percent();
    let bar_len = layout[1].width.saturating_sub(15) as usize; 
    let filled = if bar_len > 0 { (progress_percent as usize * bar_len) / 100 } else { 0 };
    
    let mut bar_str = String::new();
    for i in 0..bar_len {
        if i == filled { bar_str.push('●'); } else { bar_str.push('━'); }
    }
    
    let progress_line = Line::from(vec![
        Span::styled(format!("{:<5} ", app.elapsed_str()), Style::default().fg(Theme::MUTED)),
        Span::styled(bar_str, Style::default().fg(Theme::ACCENT_BRIGHT)),
        Span::styled(format!(" {:>5}", app.duration_str()), Style::default().fg(Theme::MUTED)),
    ]);
    frame.render_widget(Paragraph::new(progress_line).alignment(Alignment::Center), layout[1]);
    
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(Theme::ACCENT).bg(Theme::SURFACE))
        .percent(progress_percent)
        .label("");
    frame.render_widget(gauge, layout[2]);
    
    let play_icon = if app.is_playing { "⏸" } else { "▶" };
    let control_line = Line::from(vec![
        Span::styled("⏮  ⏪  ", Style::default().fg(Theme::DIM)),
        Span::styled(play_icon, Style::default().fg(Theme::ACCENT_BRIGHT)),
        Span::styled("  ⏩  ⏭", Style::default().fg(Theme::DIM)),
    ]);
    frame.render_widget(Paragraph::new(control_line).alignment(Alignment::Center), layout[4]);
}

fn draw_spectrum(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("spectrum");
    let mut data = vec![];
    for (i, &val) in app.spectrum_data.iter().enumerate() {
        data.push((format!("{}", i), val));
    }
    let ref_data: Vec<(&str, u64)> = data.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    
    let barchart = BarChart::default()
        .block(block)
        .bar_width(2)
        .bar_gap(1)
        .max(100)
        .style(Style::default().fg(Theme::ACCENT))
        .data(&ref_data);

    frame.render_widget(barchart, area);
}

fn draw_controls(frame: &mut Frame, app: &App, area: Rect) {
    let block = styled_block("controls");
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let keys_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(Rect { x: inner_area.x, y: inner_area.y, width: inner_area.width, height: 4 });

    let left_keys = vec![
        Line::from(vec![Span::styled("q/quit", Style::default().fg(Theme::ACCENT_BRIGHT)), Span::styled(" exit", Style::default().fg(Theme::MUTED))]),
        Line::from(vec![Span::styled("d     ", Style::default().fg(Theme::ACCENT_BRIGHT)), Span::styled(" dl song", Style::default().fg(Theme::MUTED))]),
        Line::from(vec![Span::styled("space ", Style::default().fg(Theme::ACCENT_BRIGHT)), Span::styled(" play/pause", Style::default().fg(Theme::MUTED))]),
        Line::from(vec![Span::styled("enter ", Style::default().fg(Theme::ACCENT_BRIGHT)), Span::styled(" play sel", Style::default().fg(Theme::MUTED))]),
    ];
    let right_keys = vec![
        Line::from(vec![Span::styled("↑↓    ", Style::default().fg(Theme::ACCENT_BRIGHT)), Span::styled(" nav", Style::default().fg(Theme::MUTED))]),
        Line::from(vec![Span::styled("←→    ", Style::default().fg(Theme::ACCENT_BRIGHT)), Span::styled(" seek", Style::default().fg(Theme::MUTED))]),
        Line::from(vec![Span::styled("+-    ", Style::default().fg(Theme::ACCENT_BRIGHT)), Span::styled(" vol", Style::default().fg(Theme::MUTED))]),
        Line::from(vec![Span::styled("s/r   ", Style::default().fg(Theme::ACCENT_BRIGHT)), Span::styled(" shuf/rep", Style::default().fg(Theme::MUTED))]),
    ];

    frame.render_widget(Paragraph::new(left_keys), keys_layout[0]);
    frame.render_widget(Paragraph::new(right_keys), keys_layout[1]);

    let sep_rect = Rect { x: inner_area.x, y: inner_area.y + 4, width: inner_area.width, height: 1 };
    frame.render_widget(Paragraph::new(Span::styled("─".repeat(inner_area.width as usize), Style::default().fg(Theme::BORDER_DIM))), sep_rect);

    let vol_rect = Rect { x: inner_area.x, y: inner_area.y + 5, width: inner_area.width, height: 1 };
    let vol_blocks = app.volume / 10;
    let mut vol_bar = String::new();
    for i in 0..10 {
        if i < vol_blocks { vol_bar.push('█'); } else { vol_bar.push('░'); }
    }
    let vol_line = Line::from(vec![
        Span::styled("vol ", Style::default().fg(Theme::MUTED)),
        Span::styled("[", Style::default().fg(Theme::DIM)),
        Span::styled(&vol_bar[0..(vol_blocks as usize * 3)], Style::default().fg(Theme::ACCENT)),
        Span::styled(&vol_bar[(vol_blocks as usize * 3)..], Style::default().fg(Theme::DIM)),
        Span::styled("] ", Style::default().fg(Theme::DIM)),
        Span::styled(format!("{}%", app.volume), Style::default().fg(Theme::GREEN)),
    ]);
    frame.render_widget(Paragraph::new(vol_line), vol_rect);

    let state_rect = Rect { x: inner_area.x, y: inner_area.y + 6, width: inner_area.width, height: 1 };
    let shuf_span = if app.shuffle { Span::styled("shuf: on  ", Style::default().fg(Theme::GREEN)) } else { Span::styled("shuf: off ", Style::default().fg(Theme::DIM)) };
    let rep_span = match app.repeat {
        RepeatMode::None => Span::styled("rep: none", Style::default().fg(Theme::DIM)),
        RepeatMode::One => Span::styled("rep: one", Style::default().fg(Theme::AMBER)),
        RepeatMode::All => Span::styled("rep: all", Style::default().fg(Theme::AMBER)),
    };
    frame.render_widget(Paragraph::new(Line::from(vec![shuf_span, rep_span])), state_rect);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let total_secs: u64 = app.tracks.iter().map(|t| t.duration_secs).sum();
    let total_mins = total_secs / 60;
    let total_rem_secs = total_secs % 60;

    let left_span = Span::styled(format!("{} tracks  ·  {}:{:02} total", app.tracks.len(), total_mins, total_rem_secs), Style::default().fg(Theme::MUTED));
    let center_span = if app.is_playing {
        Span::styled("● PLAYING", Style::default().fg(Theme::GREEN).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("○ PAUSED", Style::default().fg(Theme::DIM).add_modifier(Modifier::BOLD))
    };
    let right_span = Span::styled(format!("vol: {}%", app.volume), Style::default().fg(Theme::MUTED));

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(33)])
        .split(area);

    frame.render_widget(Paragraph::new(left_span).alignment(Alignment::Left), layout[0]);
    frame.render_widget(Paragraph::new(center_span).alignment(Alignment::Center), layout[1]);
    frame.render_widget(Paragraph::new(right_span).alignment(Alignment::Right), layout[2]);
}

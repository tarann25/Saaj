// === src/main.rs ===
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
    thread,
};
use std::sync::mpsc;

mod app;
mod theme;
mod ui;

use app::{App, AppMode, download_song};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    let (tx, rx) = mpsc::channel();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Ok(_) = rx.try_recv() {
            app.mode = AppMode::Main;
            app.download_input.clear();
            app.load_tracks();
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if app.show_splash {
                    if key.code == KeyCode::Enter {
                        app.show_splash = false;
                    }
                    continue;
                }

                match app.mode {
                    AppMode::Splash => {}, // Handled by show_splash flag
                    AppMode::DownloadPrompt => {
                        match key.code {
                            KeyCode::Esc => {
                                app.mode = AppMode::Main;
                                app.download_input.clear();
                            },
                            KeyCode::Enter => {
                                if !app.download_input.is_empty() {
                                    app.mode = AppMode::Downloading;
                                    let query = app.download_input.clone();
                                    let tx_clone = tx.clone();
                                    thread::spawn(move || {
                                        download_song(query);
                                        let _ = tx_clone.send(());
                                    });
                                }
                            },
                            KeyCode::Backspace => {
                                app.download_input.pop();
                            },
                            KeyCode::Char(c) => {
                                app.download_input.push(c);
                            },
                            _ => {}
                        }
                    },
                    AppMode::Downloading => {
                        if key.code == KeyCode::Char('q') {
                            app.should_quit = true;
                        }
                    },
                    AppMode::Main => {
                        match key.code {
                            KeyCode::Char('q') => app.should_quit = true,
                            KeyCode::Char(' ') => app.toggle_play(),
                            KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
                            KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                            KeyCode::Enter => {
                                app.current_index = app.selected;
                                app.play_current_track();
                            }
                            KeyCode::Char('d') => {
                                app.mode = AppMode::DownloadPrompt;
                            },
                            KeyCode::Left | KeyCode::Char('h') => app.seek_backward(5.0),
                            KeyCode::Right | KeyCode::Char('l') => app.seek_forward(5.0),
                            KeyCode::Char('+') | KeyCode::Char('=') => app.volume_up(),
                            KeyCode::Char('-') => app.volume_down(),
                            KeyCode::Char('s') => app.toggle_shuffle(),
                            KeyCode::Char('r') => app.cycle_repeat(),
                            _ => {}
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

//! # TUI Application
//!
//! Main TUI application with tab navigation and real-time updates.

use std::io;
use std::sync::Arc;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};

use vault_core::dashboard::{DashboardSnapshot, MetricsHistory, NodeDashboardApi};

use super::tabs::{EconomicsTab, NetworkTab, OverviewTab, StorageTab};

/// TUI Application state
pub struct TuiApp {
    /// Dashboard API for fetching metrics
    api: Arc<NodeDashboardApi>,
    /// Currently selected tab index
    current_tab: usize,
    /// Tab names
    tabs: Vec<String>,
    /// Flag to quit the application
    should_quit: bool,
    /// Current snapshot of metrics
    snapshot: Option<DashboardSnapshot>,
    /// Metrics history for sparklines
    history: Option<MetricsHistory>,
}

impl TuiApp {
    /// Create a new TUI application
    pub fn new(api: Arc<NodeDashboardApi>) -> Self {
        Self {
            api,
            current_tab: 0,
            tabs: vec![
                "Overview".to_string(),
                "Storage".to_string(),
                "Network".to_string(),
                "Economics".to_string(),
            ],
            should_quit: false,
            snapshot: None,
            history: None,
        }
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        let tick_rate = Duration::from_millis(1000); // 1 second refresh

        loop {
            // Update metrics
            self.snapshot = Some(self.api.get_snapshot().await);
            self.history = Some(self.api.get_history().await);

            // Draw UI
            terminal.draw(|f| self.ui(f))?;

            // Handle input with timeout
            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                            KeyCode::Right | KeyCode::Tab => {
                                self.current_tab = (self.current_tab + 1) % self.tabs.len();
                            }
                            KeyCode::Left | KeyCode::BackTab => {
                                self.current_tab = if self.current_tab == 0 {
                                    self.tabs.len() - 1
                                } else {
                                    self.current_tab - 1
                                };
                            }
                            KeyCode::Char('1') => self.current_tab = 0,
                            KeyCode::Char('2') => self.current_tab = 1,
                            KeyCode::Char('3') => self.current_tab = 2,
                            KeyCode::Char('4') => self.current_tab = 3,
                            _ => {}
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Render the UI
    fn ui(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header with tabs
                Constraint::Min(0),    // Main content
                Constraint::Length(1), // Footer
            ])
            .split(f.area());

        // Render header with tabs
        self.render_header(f, chunks[0]);

        // Render current tab content
        if let Some(ref snapshot) = self.snapshot {
            let history = self.history.as_ref();
            match self.current_tab {
                0 => OverviewTab::render(f, chunks[1], snapshot, history),
                1 => StorageTab::render(f, chunks[1], snapshot, history),
                2 => NetworkTab::render(f, chunks[1], snapshot, history),
                3 => EconomicsTab::render(f, chunks[1], snapshot, history),
                _ => {}
            }
        } else {
            // Loading state
            let loading = Paragraph::new("Loading metrics...")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(loading, chunks[1]);
        }

        // Render footer
        self.render_footer(f, chunks[2]);
    }

    /// Render the header with tabs
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let titles: Vec<Line> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let style = if i == self.current_tab {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(vec![Span::styled(format!(" {} ", t), style)])
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" SolanaVault Node Dashboard "),
            )
            .select(self.current_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));

        f.render_widget(tabs, area);
    }

    /// Render the footer with help text
    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let help_text = " [Tab/Arrow] Switch tabs | [1-4] Jump to tab | [q] Quit ";
        let footer = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
        f.render_widget(footer, area);
    }
}

//! # Storage Tab
//!
//! Storage tab showing detailed storage and compression metrics.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Sparkline, Table},
    Frame,
};

use vault_core::dashboard::{DashboardSnapshot, MetricsHistory};

/// Storage tab renderer
pub struct StorageTab;

impl StorageTab {
    /// Render the storage tab
    pub fn render(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot, history: Option<&MetricsHistory>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12), // Stats tables
                Constraint::Min(6),     // Sparklines
            ])
            .split(area);

        // Top section: two columns for stats
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        // Left: Storage stats
        Self::render_storage_stats(f, top_chunks[0], snapshot);

        // Right: Compression stats
        Self::render_compression_stats(f, top_chunks[1], snapshot);

        // Bottom: Sparklines
        Self::render_sparklines(f, chunks[1], history);
    }

    fn render_storage_stats(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot) {
        let storage = &snapshot.storage;

        let rows = vec![
            Row::new(vec![
                Cell::from("Total Capacity"),
                Cell::from(format!("{:.2} GB", storage.total_capacity as f64 / 1e9))
                    .style(Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Cell::from("Used Capacity"),
                Cell::from(format!("{:.2} GB", storage.used_capacity as f64 / 1e9))
                    .style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("Available"),
                Cell::from(format!("{:.2} GB", storage.available_capacity as f64 / 1e9))
                    .style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("Usage %"),
                Cell::from(format!(
                    "{:.1}%",
                    if storage.total_capacity > 0 {
                        (storage.used_capacity as f64 / storage.total_capacity as f64) * 100.0
                    } else {
                        0.0
                    }
                ))
                .style(Style::default().fg(Color::Yellow)),
            ]),
            Row::new(vec![
                Cell::from("Blocks Stored"),
                Cell::from(format!("{}", storage.blocks_stored))
                    .style(Style::default().fg(Color::Magenta)),
            ]),
            Row::new(vec![
                Cell::from("Cache Hits"),
                Cell::from(format!("{}", storage.cache_hits))
                    .style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("Cache Misses"),
                Cell::from(format!("{}", storage.cache_misses))
                    .style(Style::default().fg(Color::Red)),
            ]),
            Row::new(vec![
                Cell::from("Cache Hit Rate"),
                Cell::from(format!("{:.1}%", storage.cache_hit_rate * 100.0))
                    .style(Style::default().fg(Color::Cyan)),
            ]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Storage Statistics "),
        );

        f.render_widget(table, area);
    }

    fn render_compression_stats(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot) {
        let storage = &snapshot.storage;
        let space_saved = storage
            .total_original_bytes
            .saturating_sub(storage.total_compressed_bytes);
        let savings_percent = if storage.total_original_bytes > 0 {
            (space_saved as f64 / storage.total_original_bytes as f64) * 100.0
        } else {
            0.0
        };

        let rows = vec![
            Row::new(vec![
                Cell::from("Compression Ratio"),
                Cell::from(format!("{:.2}:1", storage.compression_ratio)).style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Row::new(vec![
                Cell::from("Original Bytes"),
                Cell::from(format!("{:.2} MB", storage.total_original_bytes as f64 / 1e6))
                    .style(Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Cell::from("Compressed Bytes"),
                Cell::from(format!(
                    "{:.2} MB",
                    storage.total_compressed_bytes as f64 / 1e6
                ))
                .style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("Space Saved"),
                Cell::from(format!("{:.2} MB", space_saved as f64 / 1e6))
                    .style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("Savings %"),
                Cell::from(format!("{:.1}%", savings_percent))
                    .style(Style::default().fg(Color::Yellow)),
            ]),
            Row::new(vec![
                Cell::from("Avg Block Size (orig)"),
                Cell::from(if storage.blocks_stored > 0 {
                    format!(
                        "{:.2} KB",
                        (storage.total_original_bytes / storage.blocks_stored) as f64 / 1e3
                    )
                } else {
                    "N/A".to_string()
                })
                .style(Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Cell::from("Avg Block Size (comp)"),
                Cell::from(if storage.blocks_stored > 0 {
                    format!(
                        "{:.2} KB",
                        (storage.total_compressed_bytes / storage.blocks_stored) as f64 / 1e3
                    )
                } else {
                    "N/A".to_string()
                })
                .style(Style::default().fg(Color::Cyan)),
            ]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Compression Statistics "),
        );

        f.render_widget(table, area);
    }

    fn render_sparklines(f: &mut Frame, area: Rect, history: Option<&MetricsHistory>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Storage usage sparkline
        let storage_data: Vec<u64> = if let Some(h) = history {
            h.as_sparkline_data("storage_used")
        } else {
            vec![]
        };
        let storage_display: Vec<u64> = if storage_data.is_empty() {
            (0..60).map(|_| 0).collect()
        } else {
            storage_data
        };

        let storage_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Storage Usage % "),
            )
            .data(&storage_display)
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(storage_sparkline, chunks[0]);

        // Compression ratio sparkline
        let ratio_data: Vec<u64> = if let Some(h) = history {
            h.as_sparkline_data("compression_ratio")
        } else {
            vec![]
        };
        let ratio_display: Vec<u64> = if ratio_data.is_empty() {
            (0..60).map(|_| 0).collect()
        } else {
            ratio_data
        };

        let ratio_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Compression Ratio "),
            )
            .data(&ratio_display)
            .style(Style::default().fg(Color::Green));

        f.render_widget(ratio_sparkline, chunks[1]);
    }
}

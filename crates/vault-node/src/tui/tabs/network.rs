//! # Network Tab
//!
//! Network tab showing peer connections, message stats, and latency.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Sparkline, Table},
    Frame,
};

use vault_core::dashboard::{DashboardSnapshot, MetricsHistory};

/// Network tab renderer
pub struct NetworkTab;

impl NetworkTab {
    /// Render the network tab
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

        // Left: Connection stats
        Self::render_connection_stats(f, top_chunks[0], snapshot);

        // Right: Message stats
        Self::render_message_stats(f, top_chunks[1], snapshot);

        // Bottom: Sparklines
        Self::render_sparklines(f, chunks[1], history);
    }

    fn render_connection_stats(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot) {
        let network = &snapshot.network;

        let peer_color = if network.connected_peers > 0 {
            Color::Green
        } else {
            Color::Red
        };

        let rows = vec![
            Row::new(vec![
                Cell::from("Connected Peers"),
                Cell::from(format!("{}", network.connected_peers)).style(
                    Style::default()
                        .fg(peer_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Row::new(vec![
                Cell::from("Total Known Peers"),
                Cell::from(format!("{}", network.total_peers))
                    .style(Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Cell::from("Connection Rate"),
                Cell::from(if network.total_peers > 0 {
                    format!(
                        "{:.1}%",
                        (network.connected_peers as f64 / network.total_peers as f64) * 100.0
                    )
                } else {
                    "N/A".to_string()
                })
                .style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("Average Latency"),
                Cell::from(format!("{:.1} ms", network.average_latency_ms))
                    .style(Style::default().fg(Color::Yellow)),
            ]),
            Row::new(vec![
                Cell::from("Bandwidth In"),
                Cell::from(format!(
                    "{:.2} KB/s",
                    network.bandwidth_in_bytes as f64 / 1024.0
                ))
                .style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("Bandwidth Out"),
                Cell::from(format!(
                    "{:.2} KB/s",
                    network.bandwidth_out_bytes as f64 / 1024.0
                ))
                .style(Style::default().fg(Color::Magenta)),
            ]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Connection Statistics "),
        );

        f.render_widget(table, area);
    }

    fn render_message_stats(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot) {
        let network = &snapshot.network;
        let consensus = &snapshot.consensus;

        let total_messages = network.messages_sent + network.messages_received;

        let rows = vec![
            Row::new(vec![
                Cell::from("Messages Sent"),
                Cell::from(format!("{}", network.messages_sent))
                    .style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("Messages Received"),
                Cell::from(format!("{}", network.messages_received))
                    .style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("Total Messages"),
                Cell::from(format!("{}", total_messages))
                    .style(Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Cell::from("Active Proposals"),
                Cell::from(format!("{}", consensus.active_proposals))
                    .style(Style::default().fg(Color::Yellow)),
            ]),
            Row::new(vec![
                Cell::from("Votes Cast"),
                Cell::from(format!("{}", consensus.votes_cast))
                    .style(Style::default().fg(Color::Magenta)),
            ]),
            Row::new(vec![
                Cell::from("Proposals Accepted"),
                Cell::from(format!("{}", consensus.proposals_accepted))
                    .style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("Proposals Rejected"),
                Cell::from(format!("{}", consensus.proposals_rejected))
                    .style(Style::default().fg(Color::Red)),
            ]),
            Row::new(vec![
                Cell::from("Reputation Score"),
                Cell::from(format!("{:.2}", consensus.reputation_score)).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Message & Consensus Statistics "),
        );

        f.render_widget(table, area);
    }

    fn render_sparklines(f: &mut Frame, area: Rect, history: Option<&MetricsHistory>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Connected peers sparkline
        let peers_data: Vec<u64> = if let Some(h) = history {
            h.as_sparkline_data("connected_peers")
        } else {
            vec![]
        };
        let peers_display: Vec<u64> = if peers_data.is_empty() {
            (0..60).map(|_| 0).collect()
        } else {
            peers_data
        };

        let peers_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Connected Peers "),
            )
            .data(&peers_display)
            .style(Style::default().fg(Color::Green));

        f.render_widget(peers_sparkline, chunks[0]);

        // Messages per second sparkline
        let msgs_data: Vec<u64> = if let Some(h) = history {
            h.as_sparkline_data("messages_per_second")
        } else {
            vec![]
        };
        let msgs_display: Vec<u64> = if msgs_data.is_empty() {
            (0..60).map(|_| 0).collect()
        } else {
            msgs_data
        };

        let msgs_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Messages/sec "),
            )
            .data(&msgs_display)
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(msgs_sparkline, chunks[1]);
    }
}

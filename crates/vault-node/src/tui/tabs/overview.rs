//! # Overview Tab
//!
//! Overview tab showing node info, storage gauge, network summary, and activity.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};

use vault_core::dashboard::{DashboardSnapshot, MetricsHistory, NodeStatus};

/// Overview tab renderer
pub struct OverviewTab;

impl OverviewTab {
    /// Render the overview tab
    pub fn render(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot, history: Option<&MetricsHistory>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Node info
                Constraint::Length(5), // Storage gauge
                Constraint::Length(5), // Network stats
                Constraint::Min(6),    // Activity sparkline
            ])
            .split(area);

        // Node info
        Self::render_node_info(f, chunks[0], snapshot);

        // Storage gauge
        Self::render_storage_gauge(f, chunks[1], snapshot);

        // Network summary
        Self::render_network_summary(f, chunks[2], snapshot);

        // Activity sparkline
        Self::render_activity(f, chunks[3], history);
    }

    fn render_node_info(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot) {
        let info = &snapshot.node_info;
        let uptime_hours = info.uptime_seconds / 3600;
        let uptime_mins = (info.uptime_seconds % 3600) / 60;
        let uptime_secs = info.uptime_seconds % 60;

        let status_color = match info.status {
            NodeStatus::Running => Color::Green,
            NodeStatus::Starting | NodeStatus::Syncing => Color::Yellow,
            NodeStatus::Degraded => Color::Red,
            NodeStatus::Stopped => Color::DarkGray,
        };

        let text = vec![
            Line::from(vec![
                Span::styled("Node ID: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &info.node_id,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Address: ", Style::default().fg(Color::Gray)),
                Span::styled(&info.address, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Version: ", Style::default().fg(Color::Gray)),
                Span::styled(&info.version, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Uptime:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}h {}m {}s", uptime_hours, uptime_mins, uptime_secs),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Status:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:?}", info.status),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Node Info "));
        f.render_widget(paragraph, area);
    }

    fn render_storage_gauge(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot) {
        let storage = &snapshot.storage;
        let percent = if storage.total_capacity > 0 {
            (storage.used_capacity as f64 / storage.total_capacity as f64) * 100.0
        } else {
            0.0
        };

        let label = format!(
            "{:.2} GB / {:.2} GB ({:.1}%) | {:.1}:1 compression | {} blocks",
            storage.used_capacity as f64 / 1_000_000_000.0,
            storage.total_capacity as f64 / 1_000_000_000.0,
            percent,
            storage.compression_ratio,
            storage.blocks_stored
        );

        let gauge_color = if percent < 70.0 {
            Color::Cyan
        } else if percent < 90.0 {
            Color::Yellow
        } else {
            Color::Red
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Storage Usage "),
            )
            .gauge_style(Style::default().fg(gauge_color).bg(Color::Black))
            .percent(percent as u16)
            .label(label);

        f.render_widget(gauge, area);
    }

    fn render_network_summary(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot) {
        let network = &snapshot.network;
        let economics = &snapshot.economics;

        let peer_color = if network.connected_peers > 0 {
            Color::Green
        } else {
            Color::Red
        };

        let text = vec![
            Line::from(vec![
                Span::styled("Peers: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", network.connected_peers),
                    Style::default().fg(peer_color),
                ),
                Span::styled(
                    format!("/{}", network.total_peers),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  |  "),
                Span::styled("Messages: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}tx {}rx", network.messages_sent, network.messages_received),
                    Style::default().fg(Color::White),
                ),
                Span::raw("  |  "),
                Span::styled("Cache: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}%", snapshot.storage.cache_hit_rate * 100.0),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Staked: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", economics.staking.own_stake),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  |  "),
                Span::styled("Pending Rewards: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", economics.staking.pending_rewards),
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  |  "),
                Span::styled("Reputation: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.2}", snapshot.consensus.reputation_score),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Network & Economics "),
        );
        f.render_widget(paragraph, area);
    }

    fn render_activity(f: &mut Frame, area: Rect, history: Option<&MetricsHistory>) {
        let data: Vec<u64> = if let Some(h) = history {
            h.as_sparkline_data("messages_per_second")
        } else {
            vec![]
        };

        // Use dummy data if no history available
        let display_data: Vec<u64> = if data.is_empty() {
            (0..60).map(|_| 0).collect()
        } else {
            data
        };

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Message Activity (last 5 min) "),
            )
            .data(&display_data)
            .style(Style::default().fg(Color::Green));

        f.render_widget(sparkline, area);
    }
}

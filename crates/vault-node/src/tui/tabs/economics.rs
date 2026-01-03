//! # Economics Tab
//!
//! Economics tab showing staking, rewards, and gateway revenue.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Sparkline, Table},
    Frame,
};

use vault_core::dashboard::{DashboardSnapshot, MetricsHistory};

/// Economics tab renderer
pub struct EconomicsTab;

impl EconomicsTab {
    /// Render the economics tab
    pub fn render(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot, history: Option<&MetricsHistory>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(12), // Stats tables
                Constraint::Min(6),     // Sparklines
            ])
            .split(area);

        // Top section: two or three columns for stats
        let top_chunks = if snapshot.economics.gateway.is_some() {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .split(chunks[0])
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[0])
        };

        // Left: Staking stats
        Self::render_staking_stats(f, top_chunks[0], snapshot);

        // Middle/Right: Reward stats
        Self::render_reward_stats(f, top_chunks[1], snapshot);

        // Right (if gateway): Gateway stats
        if let Some(gateway) = &snapshot.economics.gateway {
            if top_chunks.len() > 2 {
                Self::render_gateway_stats(f, top_chunks[2], gateway);
            }
        }

        // Bottom: Sparklines
        Self::render_sparklines(f, chunks[1], history);
    }

    fn render_staking_stats(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot) {
        let staking = &snapshot.economics.staking;

        let performance_color = if staking.performance_score >= 1.0 {
            Color::Green
        } else if staking.performance_score >= 0.5 {
            Color::Yellow
        } else {
            Color::Red
        };

        let rows = vec![
            Row::new(vec![
                Cell::from("Own Stake"),
                Cell::from(format_tokens(staking.own_stake)).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Row::new(vec![
                Cell::from("Network Total Staked"),
                Cell::from(format_tokens(staking.total_staked))
                    .style(Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Cell::from("Stake Share"),
                Cell::from(if staking.total_staked > 0 {
                    format!(
                        "{:.2}%",
                        (staking.own_stake as f64 / staking.total_staked as f64) * 100.0
                    )
                } else {
                    "N/A".to_string()
                })
                .style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("Pending Rewards"),
                Cell::from(format_tokens(staking.pending_rewards))
                    .style(Style::default().fg(Color::Green)),
            ]),
            Row::new(vec![
                Cell::from("Performance Score"),
                Cell::from(format!("{:.2}x", staking.performance_score)).style(
                    Style::default()
                        .fg(performance_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Row::new(vec![
                Cell::from("Base APY"),
                Cell::from(format!("{:.1}%", staking.base_apy * 100.0))
                    .style(Style::default().fg(Color::Magenta)),
            ]),
            Row::new(vec![
                Cell::from("Effective APY"),
                Cell::from(format!(
                    "{:.1}%",
                    staking.base_apy * staking.performance_score * 100.0
                ))
                .style(Style::default().fg(Color::Green)),
            ]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Staking "),
        );

        f.render_widget(table, area);
    }

    fn render_reward_stats(f: &mut Frame, area: Rect, snapshot: &DashboardSnapshot) {
        let rewards = &snapshot.economics.rewards;

        let rows = vec![
            Row::new(vec![
                Cell::from("Total Earned"),
                Cell::from(format_tokens(rewards.total_earned)).style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Row::new(vec![
                Cell::from("This Epoch"),
                Cell::from(format_tokens(rewards.distributed_this_epoch))
                    .style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("Epochs Completed"),
                Cell::from(format!("{}", rewards.epochs_completed))
                    .style(Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Cell::from("Avg/Epoch"),
                Cell::from(if rewards.epochs_completed > 0 {
                    format_tokens(rewards.total_earned / rewards.epochs_completed as u64)
                } else {
                    "N/A".to_string()
                })
                .style(Style::default().fg(Color::Yellow)),
            ]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Rewards "),
        );

        f.render_widget(table, area);
    }

    fn render_gateway_stats(
        f: &mut Frame,
        area: Rect,
        gateway: &vault_core::dashboard::GatewayMetricsSummary,
    ) {
        let load_color = if gateway.current_load < 0.7 {
            Color::Green
        } else if gateway.current_load < 0.9 {
            Color::Yellow
        } else {
            Color::Red
        };

        let rows = vec![
            Row::new(vec![
                Cell::from("Total Revenue"),
                Cell::from(format_tokens(gateway.total_revenue)).style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Row::new(vec![
                Cell::from("Active Clients"),
                Cell::from(format!("{}", gateway.active_clients))
                    .style(Style::default().fg(Color::Cyan)),
            ]),
            Row::new(vec![
                Cell::from("Requests Served"),
                Cell::from(format!("{}", gateway.requests_served))
                    .style(Style::default().fg(Color::White)),
            ]),
            Row::new(vec![
                Cell::from("Current Load"),
                Cell::from(format!("{:.1}%", gateway.current_load * 100.0))
                    .style(Style::default().fg(load_color)),
            ]),
            Row::new(vec![
                Cell::from("Avg Revenue/Request"),
                Cell::from(if gateway.requests_served > 0 {
                    format!("{:.2}", gateway.total_revenue as f64 / gateway.requests_served as f64)
                } else {
                    "N/A".to_string()
                })
                .style(Style::default().fg(Color::Yellow)),
            ]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Gateway Revenue "),
        );

        f.render_widget(table, area);
    }

    fn render_sparklines(f: &mut Frame, area: Rect, history: Option<&MetricsHistory>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Rewards earned sparkline
        let rewards_data: Vec<u64> = if let Some(h) = history {
            h.as_sparkline_data("rewards_earned")
        } else {
            vec![]
        };
        let rewards_display: Vec<u64> = if rewards_data.is_empty() {
            (0..60).map(|_| 0).collect()
        } else {
            rewards_data
        };

        let rewards_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Total Rewards "),
            )
            .data(&rewards_display)
            .style(Style::default().fg(Color::Green));

        f.render_widget(rewards_sparkline, chunks[0]);

        // Cache hit rate sparkline (as a proxy for efficiency)
        let cache_data: Vec<u64> = if let Some(h) = history {
            h.as_sparkline_data("cache_hit_rate")
        } else {
            vec![]
        };
        let cache_display: Vec<u64> = if cache_data.is_empty() {
            (0..60).map(|_| 0).collect()
        } else {
            cache_data
        };

        let cache_sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Cache Efficiency % "),
            )
            .data(&cache_display)
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(cache_sparkline, chunks[1]);
    }
}

/// Format token amounts for display
fn format_tokens(amount: u64) -> String {
    if amount >= 1_000_000_000 {
        format!("{:.2}B", amount as f64 / 1_000_000_000.0)
    } else if amount >= 1_000_000 {
        format!("{:.2}M", amount as f64 / 1_000_000.0)
    } else if amount >= 1_000 {
        format!("{:.2}K", amount as f64 / 1_000.0)
    } else {
        format!("{}", amount)
    }
}

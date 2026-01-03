//! # TUI Tabs
//!
//! Tab implementations for the dashboard TUI.

pub mod overview;
pub mod storage;
pub mod network;
pub mod economics;

pub use overview::OverviewTab;
pub use storage::StorageTab;
pub use network::NetworkTab;
pub use economics::EconomicsTab;

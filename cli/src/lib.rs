//! AICred CLI Library
//!
//! This library provides the core functionality for the AICred command-line interface,
//! including tag and label management commands.

pub mod commands;
pub mod output;

pub use commands::{labels, tags};

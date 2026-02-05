//! Backward compatibility re-exports.
//!
//! The scanners module has been renamed to `discovery` for clarity.
//! This module re-exports everything for backward compatibility.

#[deprecated(since = "0.2.0", note = "Use crate::discovery instead")]
pub use crate::discovery::*;

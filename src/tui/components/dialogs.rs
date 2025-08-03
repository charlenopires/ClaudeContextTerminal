//! Dialog system for modal UI components
//! 
//! This module provides a comprehensive dialog management system for the TUI,
//! supporting modal dialogs with proper layering, focus management, and keyboard navigation.
//! 
//! The system is designed to be Rust-idiomatic while maintaining compatibility with
//! the existing Component trait and event handling system.

pub mod manager;
pub mod types;
pub mod layer;
pub mod navigation;
pub mod quit;
pub mod commands;
pub mod sessions;
pub mod models;

pub use manager::DialogManager;
pub use types::*;
pub use layer::DialogLayer;
pub use navigation::DialogNavigation;
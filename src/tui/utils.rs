use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Layout utilities for TUI components
pub mod layout {
    use super::*;
    
    /// Create a centered rectangle with given width and height
    pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((area.height.saturating_sub(height)) / 2),
                Constraint::Length(height),
                Constraint::Length((area.height.saturating_sub(height)) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((area.width.saturating_sub(width)) / 2),
                Constraint::Length(width),
                Constraint::Length((area.width.saturating_sub(width)) / 2),
            ])
            .split(popup_layout[1])[1]
    }
    
    /// Create a centered rectangle with percentage of the parent area
    pub fn centered_rect_percent(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}
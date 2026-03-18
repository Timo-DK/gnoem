/// Renderer trait, layout helpers, and shared action types for the TUI.
///
/// This module defines the interface that any concrete renderer (ratatui,
/// plain-text, test-double, …) must implement, along with the `CubicleLayout`
/// helper that converts terminal dimensions into per-cubicle rectangles.
use crate::office::Office;

// ---------------------------------------------------------------------------
// AppAction
// ---------------------------------------------------------------------------

/// User-driven actions that any renderer can raise.
#[derive(Debug, Clone, PartialEq)]
pub enum AppAction {
    Quit,
    TogglePause,
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

/// Abstraction over the actual terminal backend.
///
/// Implementors are responsible for drawing the current `Office` state and
/// translating raw input events into `AppAction`s.
pub trait Renderer {
    /// Render the current state of the office to the terminal (or test buffer).
    fn render(&mut self, office: &Office) -> Result<(), Box<dyn std::error::Error>>;

    /// Poll for a pending input event.  Returns `None` when no event is
    /// available, so the caller can call this in a non-blocking loop.
    fn handle_input(&mut self) -> Option<AppAction>;

    /// Returns `true` once the renderer has been asked to shut down (e.g. the
    /// user pressed `q`).
    fn should_quit(&self) -> bool;
}

// ---------------------------------------------------------------------------
// CubicleLayout
// ---------------------------------------------------------------------------

/// Calculates how many cubicles fit in the terminal and where each one sits.
///
/// Every cubicle is `CUBICLE_W` × `CUBICLE_H` characters.  The layout is a
/// simple uniform grid; any leftover space on the right / bottom is unused.
pub struct CubicleLayout {
    /// Number of cubicle columns that fit.
    pub cols: u16,
    /// Number of cubicle rows that fit.
    pub rows: u16,
    /// Width of each cubicle in terminal columns.
    pub cubicle_width: u16,
    /// Height of each cubicle in terminal rows.
    pub cubicle_height: u16,
}

/// Fixed cubicle width (characters).
const CUBICLE_W: u16 = 32;
/// Fixed cubicle height (lines).
const CUBICLE_H: u16 = 16;

impl CubicleLayout {
    /// Compute a layout that fits inside `terminal_width` × `terminal_height`.
    ///
    /// Always produces at least one column and one row.
    pub fn calculate(terminal_width: u16, terminal_height: u16) -> Self {
        let cols = (terminal_width / CUBICLE_W).max(1);
        let rows = (terminal_height / CUBICLE_H).max(1);
        Self {
            cols,
            rows,
            cubicle_width: CUBICLE_W,
            cubicle_height: CUBICLE_H,
        }
    }

    /// Return the `(x, y, width, height)` rectangle for the `index`-th cubicle
    /// in reading order (left-to-right, top-to-bottom).
    ///
    /// Callers are responsible for not requesting an index beyond the grid
    /// capacity; out-of-bounds indices will produce coordinates outside the
    /// visible terminal.
    pub fn cubicle_rect(&self, index: usize) -> (u16, u16, u16, u16) {
        let col = (index as u16) % self.cols;
        let row = (index as u16) / self.cols;
        let x = col * self.cubicle_width;
        let y = row * self.cubicle_height;
        (x, y, self.cubicle_width, self.cubicle_height)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // CubicleLayout::calculate
    // -----------------------------------------------------------------------

    #[test]
    fn calculate_fits_two_columns_in_wide_terminal() {
        // 80 columns wide: 80 / 32 = 2 columns, remainder ignored
        let layout = CubicleLayout::calculate(80, 48);
        assert_eq!(layout.cols, 2);
        assert_eq!(layout.rows, 3);
        assert_eq!(layout.cubicle_width, 32);
        assert_eq!(layout.cubicle_height, 16);
    }

    #[test]
    fn calculate_fits_three_columns_in_very_wide_terminal() {
        // 120 / 32 = 3 (remainder 24)
        let layout = CubicleLayout::calculate(120, 16);
        assert_eq!(layout.cols, 3);
        assert_eq!(layout.rows, 1);
    }

    #[test]
    fn calculate_minimum_one_column_in_narrow_terminal() {
        // Even if the terminal is narrower than one cubicle, we still get 1 col.
        let layout = CubicleLayout::calculate(10, 10);
        assert_eq!(layout.cols, 1);
        assert_eq!(layout.rows, 1);
    }

    #[test]
    fn calculate_minimum_one_row_in_short_terminal() {
        let layout = CubicleLayout::calculate(32, 5);
        assert_eq!(layout.cols, 1);
        assert_eq!(layout.rows, 1);
    }

    #[test]
    fn calculate_zero_width_and_height_gives_one_by_one_grid() {
        // Degenerate case: terminal reports 0 dimensions.
        let layout = CubicleLayout::calculate(0, 0);
        assert_eq!(layout.cols, 1);
        assert_eq!(layout.rows, 1);
    }

    #[test]
    fn calculate_exact_fit_for_single_cubicle_terminal() {
        let layout = CubicleLayout::calculate(32, 16);
        assert_eq!(layout.cols, 1);
        assert_eq!(layout.rows, 1);
    }

    #[test]
    fn calculate_large_terminal_produces_many_cubicles() {
        // 320 × 160: 10 cols × 10 rows
        let layout = CubicleLayout::calculate(320, 160);
        assert_eq!(layout.cols, 10);
        assert_eq!(layout.rows, 10);
    }

    // -----------------------------------------------------------------------
    // CubicleLayout::cubicle_rect
    // -----------------------------------------------------------------------

    #[test]
    fn cubicle_rect_first_cubicle_is_at_origin() {
        let layout = CubicleLayout::calculate(96, 32); // 3 cols × 2 rows
        let (x, y, w, h) = layout.cubicle_rect(0);
        assert_eq!((x, y, w, h), (0, 0, 32, 16));
    }

    #[test]
    fn cubicle_rect_second_in_first_row_advances_x() {
        let layout = CubicleLayout::calculate(96, 32);
        let (x, y, w, h) = layout.cubicle_rect(1);
        assert_eq!((x, y, w, h), (32, 0, 32, 16));
    }

    #[test]
    fn cubicle_rect_first_in_second_row_advances_y() {
        let layout = CubicleLayout::calculate(96, 32); // 3 cols
        let (x, y, w, h) = layout.cubicle_rect(3);
        assert_eq!((x, y, w, h), (0, 16, 32, 16));
    }

    #[test]
    fn cubicle_rect_arbitrary_index_wraps_correctly() {
        // 4 cols: index 6 → col=2, row=1 → x=64, y=16
        let layout = CubicleLayout::calculate(128, 48); // 4 cols × 3 rows
        let (x, y, w, h) = layout.cubicle_rect(6);
        assert_eq!((x, y, w, h), (64, 16, 32, 16));
    }

    #[test]
    fn cubicle_rect_in_one_by_one_grid_is_always_at_origin() {
        // Tiny terminal → 1×1 grid; every "index" maps to (0,0).
        let layout = CubicleLayout::calculate(1, 1);
        let (x, y, w, h) = layout.cubicle_rect(0);
        assert_eq!((x, y, w, h), (0, 0, 32, 16));
    }

    // -----------------------------------------------------------------------
    // AppAction
    // -----------------------------------------------------------------------

    #[test]
    fn app_action_quit_eq() {
        assert_eq!(AppAction::Quit, AppAction::Quit);
        assert_ne!(AppAction::Quit, AppAction::TogglePause);
    }

    #[test]
    fn app_action_clone_and_debug() {
        let a = AppAction::TogglePause;
        let b = a.clone();
        assert_eq!(a, b);
        // Debug should not panic.
        let _ = format!("{:?}", b);
    }
}

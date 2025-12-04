//! Terminal output utilities for show commands.
//!
//! Provides colors, box drawing, and progress bar formatting
//! with automatic TTY detection for graceful fallback.

pub mod change;
pub mod dashboard;
pub mod spec;

use std::io::IsTerminal;

/// Standard header width for box drawing.
pub const HEADER_WIDTH: usize = 61;

/// Check if stdout is a TTY (terminal).
/// Returns false when output is piped or redirected.
pub fn is_tty() -> bool {
    std::io::stdout().is_terminal()
}

/// Apply ANSI color code to text.
/// Returns plain text if stdout is not a TTY.
pub fn color(text: &str, code: &str) -> String {
    if is_tty() {
        format!("\x1b[{}m{}\x1b[0m", code, text)
    } else {
        text.to_string()
    }
}

/// Internal helper that always applies color (for testing color logic).
#[cfg(test)]
fn color_always(text: &str, code: &str) -> String {
    format!("\x1b[{}m{}\x1b[0m", code, text)
}

// ANSI color codes
const CYAN_BOLD: &str = "1;36";
const GREEN: &str = "32";
const YELLOW: &str = "33";
const RED: &str = "31";
const BLUE: &str = "34";
const DIM: &str = "2";
const BOLD: &str = "1";

/// Cyan bold text - for headers and names.
pub fn cyan_bold(text: &str) -> String {
    color(text, CYAN_BOLD)
}

/// Green text - for ADDED status and success messages.
pub fn green(text: &str) -> String {
    color(text, GREEN)
}

/// Yellow text - for MODIFIED status and labels.
pub fn yellow(text: &str) -> String {
    color(text, YELLOW)
}

/// Red text - for REMOVED status and errors.
pub fn red(text: &str) -> String {
    color(text, RED)
}

/// Blue text - for RENAMED status.
pub fn blue(text: &str) -> String {
    color(text, BLUE)
}

/// Dim text - for counts and scenarios.
pub fn dim(text: &str) -> String {
    color(text, DIM)
}

/// Bold text - for emphasis.
pub fn bold(text: &str) -> String {
    color(text, BOLD)
}

/// Create a progress bar visualization.
///
/// # Arguments
/// * `completed` - Number of completed items
/// * `total` - Total number of items
///
/// # Returns
/// A 20-character wide progress bar using block characters.
pub fn progress_bar(completed: usize, total: usize) -> String {
    let width = 20;
    let filled = if total > 0 {
        (completed * width) / total
    } else {
        0
    };
    let empty = width - filled;
    format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

/// Create a colored progress bar visualization.
///
/// # Arguments
/// * `completed` - Number of completed items
/// * `total` - Total number of items
///
/// # Returns
/// A 20-character wide progress bar with green filled and dim empty.
pub fn colored_progress_bar(completed: usize, total: usize) -> String {
    let width = 20;
    let filled = if total > 0 {
        (completed * width) / total
    } else {
        0
    };
    let empty = width - filled;

    let filled_part = "\u{2588}".repeat(filled);
    let empty_part = "\u{2591}".repeat(empty);

    format!("{}{}", green(&filled_part), dim(&empty_part))
}

/// Create a box header with title.
///
/// # Arguments
/// * `title` - The title text to display
/// * `width` - The total width of the box (including borders)
///
/// # Returns
/// A three-line string with top border, title row, and bottom border.
///
/// ```text
/// +-----------------------------------------------------------+
/// | Title                                                      |
/// +-----------------------------------------------------------+
/// ```
pub fn box_header(title: &str, width: usize) -> String {
    // Ensure minimum width for borders and padding
    let min_width = title.len() + 4; // 2 for borders, 2 for padding
    let actual_width = width.max(min_width);

    // Calculate inner width (excluding border characters)
    let inner_width = actual_width - 2;

    // Create horizontal border (using box drawing characters)
    let horizontal = "\u{2500}".repeat(inner_width);

    // Create title line with padding
    let title_padding = inner_width.saturating_sub(title.len() + 1);
    let title_line = format!(" {}{}", title, " ".repeat(title_padding));

    format!(
        "\u{250C}{}\u{2510}\n\u{2502}{}\u{2502}\n\u{2514}{}\u{2518}",
        horizontal, title_line, horizontal
    )
}

/// Create a centered box header with title.
///
/// # Arguments
/// * `title` - The title text to display (will be centered)
/// * `width` - The total width of the box (including borders)
///
/// # Returns
/// A three-line string with centered title using Unicode box drawing.
pub fn centered_box_header(title: &str, width: usize) -> String {
    let title_len = title.len();
    let inner_width = width - 2; // Exclude border characters

    // Calculate padding for centering
    let total_padding = inner_width.saturating_sub(title_len);
    let left_padding = total_padding / 2;
    let right_padding = total_padding - left_padding;

    // Create horizontal border
    let horizontal = "\u{2500}".repeat(inner_width);

    // Create title line with centering
    let title_line = format!(
        "{}{}{}",
        " ".repeat(left_padding),
        cyan_bold(title),
        " ".repeat(right_padding)
    );

    format!(
        "\u{250C}{}\u{2510}\n\u{2502}{}\u{2502}\n\u{2514}{}\u{2518}",
        horizontal, title_line, horizontal
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test progress_bar with various values
    #[test]
    fn test_progress_bar_zero_of_ten() {
        let bar = progress_bar(0, 10);
        assert_eq!(bar.chars().count(), 20);
        assert_eq!(bar, "\u{2591}".repeat(20));
    }

    #[test]
    fn test_progress_bar_five_of_ten() {
        let bar = progress_bar(5, 10);
        assert_eq!(bar.chars().count(), 20);
        // 5/10 = 50% = 10 filled blocks
        let expected = format!("{}{}", "\u{2588}".repeat(10), "\u{2591}".repeat(10));
        assert_eq!(bar, expected);
    }

    #[test]
    fn test_progress_bar_ten_of_ten() {
        let bar = progress_bar(10, 10);
        assert_eq!(bar.chars().count(), 20);
        assert_eq!(bar, "\u{2588}".repeat(20));
    }

    #[test]
    fn test_progress_bar_zero_total_edge_case() {
        // Edge case: 0/0 should not panic and should return empty bar
        let bar = progress_bar(0, 0);
        assert_eq!(bar.chars().count(), 20);
        assert_eq!(bar, "\u{2591}".repeat(20));
    }

    #[test]
    fn test_progress_bar_partial_values() {
        // 2/5 = 40% = 8 filled blocks
        let bar = progress_bar(2, 5);
        assert_eq!(bar.chars().count(), 20);
        let expected = format!("{}{}", "\u{2588}".repeat(8), "\u{2591}".repeat(12));
        assert_eq!(bar, expected);
    }

    // Test box_header formatting
    #[test]
    fn test_box_header_basic() {
        let header = box_header("Test", 20);
        let lines: Vec<&str> = header.lines().collect();
        assert_eq!(lines.len(), 3);

        // Check top border starts and ends correctly
        assert!(lines[0].starts_with('\u{250C}')); // top-left corner
        assert!(lines[0].ends_with('\u{2510}')); // top-right corner

        // Check title line
        assert!(lines[1].starts_with('\u{2502}')); // left border
        assert!(lines[1].ends_with('\u{2502}')); // right border
        assert!(lines[1].contains("Test"));

        // Check bottom border
        assert!(lines[2].starts_with('\u{2514}')); // bottom-left corner
        assert!(lines[2].ends_with('\u{2518}')); // bottom-right corner
    }

    #[test]
    fn test_box_header_minimum_width() {
        // Title "Hello" needs at least 9 chars (5 + 2 borders + 2 padding)
        let header = box_header("Hello", 5); // Request width smaller than minimum
        let lines: Vec<&str> = header.lines().collect();

        // Should still contain the title
        assert!(lines[1].contains("Hello"));
    }

    #[test]
    fn test_box_header_wide() {
        let header = box_header("Title", 60);
        let lines: Vec<&str> = header.lines().collect();

        // All lines should have same visual width
        assert_eq!(lines[0].chars().count(), lines[2].chars().count());
    }

    #[test]
    fn test_box_header_consistent_width() {
        let header1 = box_header("Short", 61);
        let header2 = box_header("A much longer title here", 61);

        let lines1: Vec<&str> = header1.lines().collect();
        let lines2: Vec<&str> = header2.lines().collect();

        // Both should have the same width when using same width parameter
        assert_eq!(lines1[0].chars().count(), lines2[0].chars().count());
        assert_eq!(lines1[1].chars().count(), lines2[1].chars().count());
        assert_eq!(lines1[2].chars().count(), lines2[2].chars().count());
    }

    #[test]
    fn test_centered_box_header() {
        let header = centered_box_header("Test Title", 61);
        let lines: Vec<&str> = header.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with('\u{250C}'));
        assert!(lines[0].ends_with('\u{2510}'));
        assert!(lines[2].starts_with('\u{2514}'));
        assert!(lines[2].ends_with('\u{2518}'));
    }

    #[test]
    fn test_header_width_constant() {
        assert_eq!(HEADER_WIDTH, 61);
    }

    // Test color functions via color_always helper
    #[test]
    fn test_color_always_applies_codes() {
        let result = color_always("test", "32");
        assert_eq!(result, "\x1b[32mtest\x1b[0m");
    }

    #[test]
    fn test_color_always_cyan_bold() {
        let result = color_always("header", CYAN_BOLD);
        assert_eq!(result, "\x1b[1;36mheader\x1b[0m");
    }

    #[test]
    fn test_color_always_green() {
        let result = color_always("added", GREEN);
        assert_eq!(result, "\x1b[32madded\x1b[0m");
    }

    #[test]
    fn test_color_always_yellow() {
        let result = color_always("modified", YELLOW);
        assert_eq!(result, "\x1b[33mmodified\x1b[0m");
    }

    #[test]
    fn test_color_always_red() {
        let result = color_always("removed", RED);
        assert_eq!(result, "\x1b[31mremoved\x1b[0m");
    }

    #[test]
    fn test_color_always_blue() {
        let result = color_always("renamed", BLUE);
        assert_eq!(result, "\x1b[34mrenamed\x1b[0m");
    }

    #[test]
    fn test_color_always_dim() {
        let result = color_always("count", DIM);
        assert_eq!(result, "\x1b[2mcount\x1b[0m");
    }

    #[test]
    fn test_color_always_bold() {
        let result = color_always("emphasis", BOLD);
        assert_eq!(result, "\x1b[1memphasis\x1b[0m");
    }

    // Test that color() function logic works correctly
    #[test]
    fn test_color_function_structure() {
        // Test that the function doesn't panic and returns a string
        let result = color("test", "32");
        // Result will either be plain "test" or with ANSI codes
        assert!(result.contains("test"));
    }

    // Test is_tty exists and returns a boolean
    #[test]
    fn test_is_tty_returns_bool() {
        // In test environment, stdout is typically not a TTY
        let _result: bool = is_tty();
        // Just verify it doesn't panic and returns a bool
    }

    // Test all color helper functions don't panic
    #[test]
    fn test_all_color_helpers_work() {
        let _ = cyan_bold("test");
        let _ = green("test");
        let _ = yellow("test");
        let _ = red("test");
        let _ = blue("test");
        let _ = dim("test");
        let _ = bold("test");
    }

    #[test]
    fn test_colored_progress_bar() {
        let bar = colored_progress_bar(5, 10);
        // Should contain the progress bar characters
        assert!(bar.contains('\u{2588}') || bar.contains('\u{2591}') || bar.contains("\x1b["));
    }
}

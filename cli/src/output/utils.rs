//! Utility functions for CLI output formatting.
//!
//! Provides helper functions for consistent formatting across different
//! output modules.

use aicred_core::models::model_registry::ModelCapabilities;

/// Format model capabilities as a compact string.
///
/// # Arguments
///
/// * `caps` - The model capabilities to format
/// * `verbose` - If true, use full capability names, otherwise use abbreviations
///
/// # Returns
///
/// A comma-separated string of capability indicators
pub fn format_capabilities(caps: &ModelCapabilities, verbose: bool) -> String {
    let mut parts = Vec::new();

    if caps.text {
        parts.push(if verbose { "text".to_string() } else { "T".to_string() });
    }
    if caps.vision {
        parts.push(if verbose {
            "vision".to_string()
        } else {
            "V".to_string()
        });
    }
    if caps.code {
        parts.push(if verbose { "code".to_string() } else { "C".to_string() });
    }
    if caps.function_calling {
        parts.push(if verbose {
            "func".to_string()
        } else {
            "F".to_string()
        });
    }
    if caps.streaming {
        parts.push(if verbose {
            "stream".to_string()
        } else {
            "S".to_string()
        });
    }
    if caps.json_mode {
        parts.push(if verbose {
            "json".to_string()
        } else {
            "J".to_string()
        });
    }
    if caps.audio_in {
        parts.push(if verbose {
            "audio-in".to_string()
        } else {
            "AI".to_string()
        });
    }
    if caps.audio_out {
        parts.push(if verbose {
            "audio-out".to_string()
        } else {
            "AO".to_string()
        });
    }

    if parts.is_empty() {
        if verbose {
            "none".to_string()
        } else {
            "-".to_string()
        }
    } else {
        parts.join(if verbose { ", " } else { " " })
    }
}

/// Truncate a string to a maximum length.
///
/// # Arguments
///
/// * `s` - The string to truncate
/// * `max_len` - Maximum length in characters
///
/// # Returns
///
/// The truncated string with "..." appended if truncated
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
    format!("{}...", truncated)
}

/// Format a price in USD per token to a human-readable string.
///
/// # Arguments
///
/// * `price` - Price per token in USD
///
/// # Returns
///
/// A formatted string like "$0.001/1K tokens"
pub fn format_price_per_token(price: f64) -> String {
    if price == 0.0 {
        "Free".to_string()
    } else {
        // Scale to per 1K or 1M tokens for readability
        if price < 0.000001 {
            format!("${:.6}/token", price)
        } else if price < 0.001 {
            format!("${:.6}/1K tokens", price * 1000.0)
        } else {
            format!("${:.4}/1K tokens", price * 1000.0)
        }
    }
}

/// Format a number with thousands separators.
///
/// # Arguments
///
/// * `num` - The number to format
///
/// # Returns
///
/// A string with comma separators (e.g., "1,234,567")
pub fn format_number(num: u64) -> String {
    let s = num.to_string();
    let chars: Vec<char> = s.chars().collect();
    let mut result = Vec::new();
    let mut count = 0;

    for c in chars.iter().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
        count += 1;
    }

    result.into_iter().rev().collect()
}

/// Format a context length for display.
///
/// # Arguments
///
/// * `context_length` - Context length in tokens
///
/// # Returns
///
/// A formatted string (e.g., "128K" or "1M tokens")
pub fn format_context_length(context_length: u32) -> String {
    if context_length >= 1_000_000 {
        format!("{}M", context_length / 1_000_000)
    } else if context_length >= 1000 {
        format!("{}K", context_length / 1000)
    } else {
        context_length.to_string()
    }
}

/// Format a file size in bytes to human-readable form.
///
/// # Arguments
///
/// * `bytes` - Size in bytes
///
/// # Returns
///
/// A formatted string like "1.5 MB" or "2.3 GB"
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Create a horizontal rule with specified character and length.
///
/// # Arguments
///
/// * `ch` - The character to use (typically '─' or '=')
/// * `length` - Length of the rule
///
/// # Returns
///
/// A string containing the horizontal rule
pub fn hr(ch: char, length: usize) -> String {
    ch.to_string().repeat(length)
}

/// Wrap text to a specified width.
///
/// # Arguments
///
/// * `text` - The text to wrap
/// * `width` - Maximum line width
///
/// # Returns
///
/// A vector of wrapped lines
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_length = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        if current_length == 0 {
            // First word on line
            current_line.push_str(word);
            current_length = word_len;
        } else if current_length + 1 + word_len <= width {
            // Word fits on current line
            current_line.push(' ');
            current_line.push_str(word);
            current_length += 1 + word_len;
        } else {
            // Word doesn't fit, start new line
            result.push(current_line);
            current_line = word.to_string();
            current_length = word_len;
        }
    }

    if !current_line.is_empty() {
        result.push(current_line);
    }

    result
}

/// Calculate estimated cost for a given token usage.
///
/// # Arguments
///
/// * `input_tokens` - Number of input tokens
/// * `output_tokens` - Number of output tokens
/// * `input_price` - Price per input token in USD
/// * `output_price` - Price per output token in USD
///
/// # Returns
///
/// Total cost in USD
pub fn calculate_cost(
    input_tokens: u32,
    output_tokens: u32,
    input_price: f64,
    output_price: f64,
) -> f64 {
    let input_cost = input_tokens as f64 * input_price;
    let output_cost = output_tokens as f64 * output_price;
    input_cost + output_cost
}

/// Format a duration in seconds to human-readable form.
///
/// # Arguments
///
/// * `seconds` - Duration in seconds
///
/// # Returns
///
/// A formatted string like "5m 30s" or "2h 15m"
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = Vec::new();

    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{}s", secs));
    }

    parts.join(" ")
}

/// Get a colored checkmark or X for boolean values.
///
/// # Arguments
///
/// * `value` - Boolean value
///
/// # Returns
///
/// A colored indicator (✓ or ✗)
pub fn bool_indicator_colored(value: bool) -> &'static str {
    if value {
        "✓"
    } else {
        "✗"
    }
}

/// Pad a string to a specified width.
///
/// # Arguments
///
/// * `s` - The string to pad
/// * `width` - Target width
/// * `align_left` - If true, pad on the right; otherwise pad on the left
///
/// # Returns
///
/// The padded string
pub fn pad_string(s: &str, width: usize, align_left: bool) -> String {
    let len = s.chars().count();
    if len >= width {
        return s.to_string();
    }

    let padding = width - len;
    let pad_str = " ".repeat(padding);

    if align_left {
        format!("{}{}", s, pad_str)
    } else {
        format!("{}{}", pad_str, s)
    }
}

/// Create a table row from a vector of cells.
///
/// # Arguments
///
/// * `cells` - Vector of cell strings
/// * `widths` - Vector of column widths
/// * `separator` - String to use between columns
///
/// # Returns
///
/// A formatted table row string
pub fn table_row(cells: Vec<&str>, widths: &[usize], separator: &str) -> String {
    let mut result = String::new();

    for (i, (cell, &width)) in cells.iter().zip(widths.iter()).enumerate() {
        if i > 0 {
            result.push_str(separator);
        }
        result.push_str(&pad_string(cell, width, true));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_string_no_truncation() {
        let result = truncate_string("short", 20);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_string_with_truncation() {
        let result = truncate_string("this is a very long string", 10);
        assert_eq!(result.len(), 10);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(999), "999");
    }

    #[test]
    fn test_format_context_length() {
        assert_eq!(format_context_length(128000), "128K");
        assert_eq!(format_context_length(1000000), "1M");
        assert_eq!(format_context_length(999), "999");
    }

    #[test]
    fn test_format_file_size() {
        assert!(format_file_size(1024).contains("KB"));
        assert!(format_file_size(1024 * 1024).contains("MB"));
    }

    #[test]
    fn test_wrap_text() {
        let result = wrap_text("hello world this is a test", 10);
        assert!(result.len() > 1); // Should wrap to multiple lines
    }

    #[test]
    fn test_calculate_cost() {
        let cost = calculate_cost(1000, 500, 0.000001, 0.000002);
        assert_eq!(cost, 0.002);
    }

    #[test]
    fn test_format_duration() {
        assert!(format_duration(3665).contains("1h"));
        assert!(format_duration(125).contains("2m"));
        assert!(format_duration(30).contains("30s"));
    }

    #[test]
    fn test_pad_string() {
        assert_eq!(pad_string("test", 10, true), "test      ");
        assert_eq!(pad_string("test", 10, false), "      test");
    }
}

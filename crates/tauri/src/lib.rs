//! Claude Config Manager - GUI Application
//!
//! Tauri-based desktop application for managing Claude Code configurations.

// Tauri commands will be added here
// mod commands;

// Re-exports
pub use tauri;

/// Tauri command handler placeholder
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(
            greet("World"),
            "Hello, World! You've been greeted from Rust!"
        );
    }
}

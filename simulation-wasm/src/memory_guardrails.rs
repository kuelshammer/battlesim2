//! Memory guardrails for safe simulation execution
//!
//! This module provides protections against out-of-memory (OOM) errors:
//! - Automatic detection and disabling of full event logging for large simulations
//! - User-friendly panic hooks for OOM errors
//! - Configuration thresholds for memory safety

use std::panic;

/// Maximum iterations before forcing lightweight mode (no full event logs)
const MAX_ITERATIONS_FOR_FULL_LOGGING: usize = 1000;

/// Initialize memory guardrails for WASM environment
///
/// This sets up panic hooks to provide user-friendly error messages
/// when out-of-memory errors occur, instead of cryptic "unreachable" messages.
pub fn init_memory_guardrails() {
    // Only set up panic hooks for WASM target
    #[cfg(target_arch = "wasm32")]
    {
        set_wasm_panic_hook();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // For native/testing, use a simpler panic hook
        set_native_panic_hook();
    }
}

/// Check if full logging should be disabled based on iteration count
///
/// Returns `true` if iterations exceed the safe threshold and lightweight mode
/// should be enforced to prevent OOM errors.
///
/// # Arguments
/// * `iterations` - Number of simulation iterations requested
///
/// # Returns
/// `true` if lightweight mode should be forced, `false` otherwise
pub fn should_force_lightweight_mode(iterations: usize) -> bool {
    iterations > MAX_ITERATIONS_FOR_FULL_LOGGING
}

/// Get a user-friendly message explaining why full logging was disabled
pub fn get_lightweight_mode_message(iterations: usize) -> String {
    format!(
        "Large simulation detected ({} iterations). Full event logging has been automatically disabled to prevent out-of-memory errors. \
         You'll receive aggregated statistics instead of detailed event logs. \
         Use smaller iteration counts (≤{}) if you need full event playback.",
        iterations,
        MAX_ITERATIONS_FOR_FULL_LOGGING
    )
}

/// Set up panic hook for WASM with user-friendly OOM messages
#[cfg(target_arch = "wasm32")]
fn set_wasm_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown error".to_string()
        };

        // Check for common OOM indicators
        let is_oom = payload.contains("memory")
            || payload.contains("stack overflow")
            || payload.contains("allocation")
            || payload.contains("heap")
            || payload.contains("out of memory")
            || payload.contains("unreachable");

        let message = if is_oom {
            format!(
                "💾 Simulation ran out of memory. Try reducing the number of iterations or \
                 using the built-in Two-Pass system which automatically handles large simulations efficiently. \
                 Error: {}",
                payload
            )
        } else {
            format!("Simulation error: {}", payload)
        };

        // Log to console for WASM
        web_sys::console::error_1(&message.into());
    }));
}

/// Set up panic hook for native/testing environments
#[cfg(not(target_arch = "wasm32"))]
fn set_native_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown error".to_string()
        };

        // Check for common OOM indicators
        let is_oom = payload.contains("memory")
            || payload.contains("stack overflow")
            || payload.contains("allocation")
            || payload.contains("heap");

        let message = if is_oom {
            format!(
                "💾 Simulation ran out of memory. Try reducing the number of iterations or \
                 using the built-in Two-Pass system which automatically handles large simulations efficiently. \
                 Error: {}",
                payload
            )
        } else {
            format!("Simulation error: {}", payload)
        };

        eprintln!("{}", message);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_force_lightweight_mode_under_threshold() {
        assert!(!should_force_lightweight_mode(100));
        assert!(!should_force_lightweight_mode(500));
        assert!(!should_force_lightweight_mode(1000));
    }

    #[test]
    fn test_should_force_lightweight_mode_over_threshold() {
        assert!(should_force_lightweight_mode(1001));
        assert!(should_force_lightweight_mode(5000));
        assert!(should_force_lightweight_mode(10000));
    }

    #[test]
    fn test_should_force_lightweight_mode_exact_threshold() {
        // Exactly at threshold should NOT force lightweight mode
        assert!(!should_force_lightweight_mode(MAX_ITERATIONS_FOR_FULL_LOGGING));
    }

    #[test]
    fn test_lightweight_mode_message_contains_useful_info() {
        let msg = get_lightweight_mode_message(5000);
        assert!(msg.contains("5000"));
        assert!(msg.contains("1000"));
        assert!(msg.contains("aggregated statistics"));
    }
}

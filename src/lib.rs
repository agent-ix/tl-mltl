//! Finite-trace MLTL evaluation, horizon analysis, and runtime-monitor interoperability.

/// Placeholder entry point.
pub fn hello() -> &'static str {
    "hello from tl_mltl"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_returns_greeting() {
        assert!(hello().contains("tl_mltl"));
    }
}

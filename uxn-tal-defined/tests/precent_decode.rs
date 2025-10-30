#[cfg(test)]
mod tests {
    use uxn_tal_defined::percent_decode_or_original;

    #[test]
    fn no_percent_fast_path() {
        assert_eq!(percent_decode_or_original("plain"), "plain");
    }

    #[test]
    fn simple_decodes() {
        assert_eq!(
            percent_decode_or_original("Hello%20World%21"),
            "Hello World!"
        );
        assert_eq!(percent_decode_or_original("%E2%9C%93"), "✓");
        assert_eq!(percent_decode_or_original("Rust%3A%20safe"), "Rust: safe");
    }

    #[test]
    fn malformed_sequences_kept() {
        assert_eq!(percent_decode_or_original("%"), "%");
        assert_eq!(percent_decode_or_original("%G1"), "%G1");
        assert_eq!(percent_decode_or_original("abc%ZZdef"), "abc%ZZdef");
    }

    #[test]
    fn invalid_utf8_falls_back() {
        // "%FF" decodes to 0xFF which is invalid UTF-8; we fall back.
        assert_eq!(percent_decode_or_original("%FF"), "%FF");
    }
}

// Enhanced ProtocolParser that includes git support
use uxn_tal_defined::v1::ProtocolParseResult;

pub struct ProtocolParser;

impl ProtocolParser {
    /// Parse a uxntal protocol URL with automatic git URL enhancement
    /// This shadows the base ProtocolParser::parse to provide git support automatically
    pub fn parse(raw_url: &str) -> ProtocolParseResult {
        crate::parse_uxntal_url(raw_url)
    }
}

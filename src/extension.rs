// The trait for parsing custom extensions in a markup language
pub trait ExtensionParser {
    type Output;
    // Parse the content inside brackets and return an optional custom output
    fn parse_bracket(&self, content: &str) -> Option<Self::Output>;
}

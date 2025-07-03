use harper_core::Lrc;
use harper_core::parsers::{Markdown, MarkdownOptions, Parser};

/// A Harper parser for Git commit files
#[derive(Clone)]
pub struct GitCommitParser {
    inner: Lrc<dyn Parser>,
}

impl GitCommitParser {
    pub fn new(parser: Lrc<dyn Parser>) -> Self {
        Self { inner: parser }
    }

    pub fn new_markdown(markdown_options: MarkdownOptions) -> Self {
        Self::new(Lrc::new(Markdown::new(markdown_options)))
    }
}

impl Parser for GitCommitParser {
    /// Admittedly a somewhat naive implementation.
    /// We're going to get _something_ to work, before we polish it off.
    fn parse(&self, source: &[char]) -> Vec<harper_core::Token> {
        // Locate the first `#`
        let end = source
            .iter()
            .position(|c| *c == '#')
            .unwrap_or(source.len());

        self.inner.parse(&source[0..end])
    }
}

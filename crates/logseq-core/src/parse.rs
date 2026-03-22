use crate::ast::{Document, Item};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("not implemented")]
    NotImplemented,
}

/// Parse a log-sig markdown string into an AST.
///
/// NOTE: real implementation will follow once the log-sig grammar is specified.
pub fn parse(_input: &str) -> Result<Document, ParseError> {
    Ok(Document {
        items: vec![Item::Unknown {
            raw: "TODO".to_string(),
        }],
    })
}

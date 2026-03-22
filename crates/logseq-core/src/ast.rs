use serde::{Deserialize, Serialize};

/// Root node for a parsed log-sig markdown document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Item {
    /// Placeholder: update once log-sig grammar is defined.
    Unknown { raw: String },
}

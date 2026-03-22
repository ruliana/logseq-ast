use serde::{Deserialize, Serialize};

/// Root node for a parsed Logseq markdown page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Document {
    pub version: u32,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    /// Block UUID if present (commonly stored as `id:: <uuid>` in properties).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// TODO/DOING/DONE/NOW/LATER, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marker: Option<Marker>,

    /// Parsed properties attached to this block.
    pub properties: Vec<Property>,

    /// Inline content of the block.
    pub content: Vec<Inline>,

    /// Nested child blocks.
    pub children: Vec<Block>,

    /// Original line number in the source file (1-indexed).
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Property {
    pub key: String,
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Marker {
    Todo,
    Doing,
    Done,
    Later,
    Now,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Inline {
    Text {
        value: String,
    },
    PageRef {
        title: String,
        original: String,
    },
    BlockRef {
        uuid: String,
    },
    Tag {
        title: String,
        original: String,
    },
    Link {
        label: Vec<Inline>,
        url: String,
    },
    Embed {
        target: EmbedTarget,
    },
    CodeSpan {
        code: String,
    },

    /// Fenced code block (```lang ... ```). This is a block-level construct but we
    /// model it as an inline node inside a `Block` to preserve ordering.
    CodeBlock {
        #[serde(skip_serializing_if = "Option::is_none")]
        info: Option<String>,
        text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EmbedTarget {
    PageRef { title: String, original: String },
    BlockRef { uuid: String },
}

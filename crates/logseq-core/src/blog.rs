use crate::ast::{Block, Document, Inline, Marker, Node};

/// Render a blog-style Markdown document from a Logseq AST.
///
/// Rules are defined by conversation requirements:
/// - No frontmatter.
/// - Top-level blocks become paragraphs.
/// - Nested blocks become sentences appended to the parent paragraph.
/// - If a block (or appended child) is missing terminal punctuation, add '.'.
/// - If a block ends with ':', its children render as nested markdown lists (and all descendants stay lists).
/// - Page refs become plain text; tags, embeds, block refs are dropped.
pub fn render_blog_markdown(doc: &Document) -> String {
    let mut out = String::new();

    // Treat document as a sequence; blank lines become paragraph breaks.
    let mut first = true;
    for node in &doc.items {
        match node {
            Node::BlankLine { .. } => {
                // normalize: single blank line
                if !out.ends_with("\n\n") {
                    out.push_str("\n\n");
                }
                first = true;
            }
            Node::Block(b) => {
                if !first && !out.ends_with("\n\n") {
                    out.push_str("\n\n");
                }
                out.push_str(&render_top_level_block(b));
                first = false;
            }
        }
    }

    // trim trailing whitespace/newlines to a single newline
    while out.ends_with("\n\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }

    out
}

fn render_top_level_block(b: &Block) -> String {
    // Paragraph from this block + appended sentences from children, unless list-mode triggered.
    let mut para = render_block_text(b);

    if !ends_with_punct(&para) {
        para.push('.');
    }

    if para.trim_end().ends_with(':') {
        // list mode
        let mut out = String::new();
        out.push_str(para.trim_end());
        out.push('\n');
        out.push_str(&render_children_as_list(&b.children, 0));
        return out;
    }

    for child in &b.children {
        let mut sentence = render_block_text(child);
        if sentence.is_empty() {
            continue;
        }
        if !ends_with_punct(&sentence) {
            sentence.push('.');
        }
        para.push(' ');
        para.push_str(sentence.trim());

        // grand-children: keep appending as prose too
        append_descendants_as_sentences(child, &mut para);
    }

    para
}

fn append_descendants_as_sentences(b: &Block, para: &mut String) {
    for child in &b.children {
        let mut sentence = render_block_text(child);
        if sentence.is_empty() {
            continue;
        }
        if !ends_with_punct(&sentence) {
            sentence.push('.');
        }
        para.push(' ');
        para.push_str(sentence.trim());
        append_descendants_as_sentences(child, para);
    }
}

fn render_children_as_list(children: &[Block], indent_level: usize) -> String {
    let mut out = String::new();
    for child in children {
        let mut text = render_block_text(child);
        if !ends_with_punct(&text) {
            text.push('.');
        }

        out.push_str(&"  ".repeat(indent_level));
        out.push_str("- ");
        out.push_str(text.trim());
        out.push('\n');

        if !child.children.is_empty() {
            out.push_str(&render_children_as_list(&child.children, indent_level + 1));
        }
    }
    out
}

fn render_block_text(b: &Block) -> String {
    // marker handling: drop markers for now
    let _m: &Option<Marker> = &b.marker;

    let mut s = String::new();

    // For v1 blog output we ignore properties entirely.

    for inl in &b.content {
        match inl {
            Inline::Text { value } => s.push_str(value),
            Inline::CodeSpan { code } => {
                s.push('`');
                s.push_str(code);
                s.push('`');
            }
            Inline::Link { label, url } => {
                // Render label with inline conversion.
                let label_s = render_inlines(label);
                s.push('[');
                s.push_str(label_s.trim());
                s.push_str("](");
                s.push_str(url);
                s.push(')');
            }
            Inline::PageRef { title, .. } => s.push_str(title),
            Inline::Tag { .. } => {}
            Inline::BlockRef { .. } => {}
            Inline::Embed { .. } => {}
            Inline::CodeBlock { info, text } => {
                // Code fences: render as fenced code block, separated from prose.
                if !s.ends_with('\n') && !s.is_empty() {
                    s.push('\n');
                }
                s.push_str("```");
                if let Some(lang) = info {
                    s.push_str(lang);
                }
                s.push('\n');
                s.push_str(text);
                s.push('\n');
                s.push_str("```\n");
            }
        }
    }

    // collapse internal whitespace a bit for prose.
    s = s.replace("\n\n", "\n");
    s.trim().to_string()
}

fn render_inlines(inls: &[Inline]) -> String {
    let mut s = String::new();
    for inl in inls {
        match inl {
            Inline::Text { value } => s.push_str(value),
            Inline::CodeSpan { code } => {
                s.push('`');
                s.push_str(code);
                s.push('`');
            }
            Inline::Link { label, url } => {
                let label_s = render_inlines(label);
                s.push('[');
                s.push_str(label_s.trim());
                s.push_str("](");
                s.push_str(url);
                s.push(')');
            }
            Inline::PageRef { title, .. } => s.push_str(title),
            Inline::Tag { .. } => {}
            Inline::BlockRef { .. } => {}
            Inline::Embed { .. } => {}
            Inline::CodeBlock { .. } => {}
        }
    }
    s
}

fn ends_with_punct(s: &str) -> bool {
    let st = s.trim_end();
    st.ends_with('.') || st.ends_with('!') || st.ends_with('?') || st.ends_with(':')
}

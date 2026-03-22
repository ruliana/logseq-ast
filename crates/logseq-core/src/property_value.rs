use crate::ast::Inline;
use crate::tokenize::tokenize_inline;

/// Parse a property value into inline nodes.
///
/// Goals:
/// - Reuse Logseq inline parsing (wiki refs, tags, block refs, etc.)
/// - Additionally recognize *bare URLs* (https://...) and turn them into Link nodes.
pub fn parse_property_value(value: &str) -> Vec<Inline> {
    let nodes = tokenize_inline(value);
    split_bare_urls(nodes)
}

fn split_bare_urls(nodes: Vec<Inline>) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::new();

    for n in nodes {
        match n {
            Inline::Text { value } => {
                out.extend(split_text_with_urls(&value));
            }
            other => out.push(other),
        }
    }

    coalesce_text(out)
}

fn split_text_with_urls(s: &str) -> Vec<Inline> {
    // Very small URL recognizer: split on whitespace, keep separators.
    // We only promote tokens that begin with http:// or https://.
    let mut out: Vec<Inline> = Vec::new();
    let mut buf = String::new();

    for part in s.split_inclusive(char::is_whitespace) {
        // part includes trailing whitespace; we analyze the non-ws token.
        let token = part.trim_end_matches(char::is_whitespace);
        let ws = &part[token.len()..];

        if token.starts_with("http://") || token.starts_with("https://") {
            if !buf.is_empty() {
                out.push(Inline::Text {
                    value: std::mem::take(&mut buf),
                });
            }
            out.push(Inline::Link {
                label: vec![Inline::Text {
                    value: token.to_string(),
                }],
                url: token.to_string(),
            });
            if !ws.is_empty() {
                out.push(Inline::Text {
                    value: ws.to_string(),
                });
            }
        } else {
            buf.push_str(part);
        }
    }

    if !buf.is_empty() {
        out.push(Inline::Text { value: buf });
    }

    out
}

fn coalesce_text(nodes: Vec<Inline>) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::new();
    for n in nodes {
        match n {
            Inline::Text { value } => {
                if value.is_empty() {
                    continue;
                }
                match out.last_mut() {
                    Some(Inline::Text { value: v }) => v.push_str(&value),
                    _ => out.push(Inline::Text { value }),
                }
            }
            other => out.push(other),
        }
    }
    out
}

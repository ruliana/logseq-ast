use crate::ast::{EmbedTarget, Inline};

/// Tokenize a single Logseq block string into inline nodes.
///
/// This is intentionally conservative (aims to not mis-parse). Unknown patterns
/// fall back to `Inline::Text`.
pub fn tokenize_inline(input: &str) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::new();
    let mut i = 0;
    let s = input;

    while i < s.len() {
        // code span: `...`
        if s.as_bytes()[i] == b'`'
            && let Some(end) = s[i + 1..].find('`')
        {
            let code = &s[i + 1..i + 1 + end];
            out.push(Inline::CodeSpan {
                code: code.to_string(),
            });
            i = i + 1 + end + 1;
            continue;
        }

        // embed: {{embed ...}}
        if s[i..].starts_with("{{embed ")
            && let Some(end) = s[i..].find("}}")
        {
            let inner = s[i + "{{embed ".len()..i + end].trim();
            if let Some(target) = parse_embed_target(inner) {
                out.push(Inline::Embed { target });
                i = i + end + 2;
                continue;
            }
        }

        // labeled refs / links: [label](...)
        if s.as_bytes()[i] == b'['
            && let Some(close_bracket) = s[i + 1..].find(']')
        {
            let label_raw = &s[i + 1..i + 1 + close_bracket];
            let after = i + 1 + close_bracket + 1;
            if after < s.len()
                && s.as_bytes()[after] == b'('
                && let Some(close_paren) = find_matching_paren(s, after)
            {
                let target_raw = &s[after + 1..close_paren];

                // [label]([[Page]])
                if let Some(page) = parse_page_ref(target_raw) {
                    out.push(Inline::Link {
                        label: tokenize_inline(label_raw),
                        url: format!("[[{}]]", page.original),
                    });
                    i = close_paren + 1;
                    continue;
                }

                // [label](((uuid)))
                if let Some(uuid) = parse_block_ref(target_raw) {
                    out.push(Inline::Link {
                        label: tokenize_inline(label_raw),
                        url: format!("(({}))", uuid),
                    });
                    i = close_paren + 1;
                    continue;
                }

                // standard markdown link [label](url)
                if is_probably_url(target_raw) {
                    out.push(Inline::Link {
                        label: tokenize_inline(label_raw),
                        url: target_raw.to_string(),
                    });
                    i = close_paren + 1;
                    continue;
                }
            }
        }

        // page ref: [[...]]
        if s[i..].starts_with("[[")
            && let Some(end) = s[i + 2..].find("]]")
        {
            let inner = &s[i + 2..i + 2 + end];
            let title = normalize_page_title(inner);
            out.push(Inline::PageRef {
                title,
                original: inner.to_string(),
            });
            i = i + 2 + end + 2;
            continue;
        }

        // block ref: ((...))
        if s[i..].starts_with("((")
            && let Some(end) = s[i + 2..].find("))")
        {
            let inner = s[i + 2..i + 2 + end].trim();
            out.push(Inline::BlockRef {
                uuid: inner.to_string(),
            });
            i = i + 2 + end + 2;
            continue;
        }

        // tags: #[[...]] or #word
        if s.as_bytes()[i] == b'#' {
            // #[[multi word]]
            if s[i..].starts_with("#[[")
                && let Some(end) = s[i + 3..].find("]]")
            {
                let inner = &s[i + 3..i + 3 + end];
                out.push(Inline::Tag {
                    title: normalize_page_title(inner),
                    original: inner.to_string(),
                });
                i = i + 3 + end + 2;
                continue;
            }

            // #word (stop at whitespace or punctuation)
            let mut j = i + 1;
            while j < s.len() {
                let c = s[j..].chars().next().unwrap();
                if c.is_whitespace() {
                    break;
                }
                // stop at common punctuation that ends a tag
                if [',', '.', ';', ':', '!', '?', ')', ']', '}', '"', '\''].contains(&c) {
                    break;
                }
                j += c.len_utf8();
            }
            if j > i + 1 {
                let inner = &s[i + 1..j];
                out.push(Inline::Tag {
                    title: normalize_page_title(inner),
                    original: inner.to_string(),
                });
                i = j;
                continue;
            }
        }

        // default: emit one char as text (we will coalesce later)
        let ch = s[i..].chars().next().unwrap();
        push_text_if_needed(&mut out, &ch.to_string());
        i += ch.len_utf8();
    }

    coalesce_text(out)
}

fn parse_embed_target(inner: &str) -> Option<EmbedTarget> {
    // {{embed [[page]]}}
    if let Some(p) = parse_page_ref(inner) {
        return Some(EmbedTarget::PageRef {
            title: normalize_page_title(&p.original),
            original: p.original,
        });
    }

    // {{embed ((uuid))}}
    if let Some(uuid) = parse_block_ref(inner) {
        return Some(EmbedTarget::BlockRef { uuid });
    }

    None
}

struct ParsedPageRef {
    original: String,
}

fn parse_page_ref(s: &str) -> Option<ParsedPageRef> {
    let st = s.trim();
    if st.starts_with("[[") && st.ends_with("]]") && st.len() >= 4 {
        return Some(ParsedPageRef {
            original: st[2..st.len() - 2].to_string(),
        });
    }
    None
}

fn parse_block_ref(s: &str) -> Option<String> {
    let st = s.trim();
    if st.starts_with("((") && st.ends_with("))") && st.len() >= 4 {
        return Some(st[2..st.len() - 2].trim().to_string());
    }
    None
}

fn find_matching_paren(s: &str, open_idx: usize) -> Option<usize> {
    // `open_idx` points at '(' in `s`.
    let mut depth = 0i32;
    let mut i = open_idx;
    while i < s.len() {
        let c = s[i..].chars().next().unwrap();
        if c == '(' {
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
        i += c.len_utf8();
    }
    None
}

fn normalize_page_title(s: &str) -> String {
    // conservative normalization: trim only (Logseq is case-insensitive in many contexts,
    // but we preserve original and keep title as trimmed).
    s.trim().to_string()
}

fn is_probably_url(s: &str) -> bool {
    let st = s.trim();
    st.starts_with("http://")
        || st.starts_with("https://")
        || st.starts_with("file:")
        || st.starts_with("/")
        || st.starts_with("./")
        || st.starts_with("../")
}

fn push_text_if_needed(out: &mut Vec<Inline>, text: &str) {
    if text.is_empty() {
        return;
    }
    match out.last_mut() {
        Some(Inline::Text { value }) => value.push_str(text),
        _ => out.push(Inline::Text {
            value: text.to_string(),
        }),
    }
}

fn coalesce_text(nodes: Vec<Inline>) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::new();
    for n in nodes {
        match n {
            Inline::Text { value } => {
                if value.is_empty() {
                    continue;
                }
                push_text_if_needed(&mut out, &value);
            }
            other => out.push(other),
        }
    }
    out
}

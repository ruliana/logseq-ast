use crate::ast::{EmbedTarget, Inline};

/// Tokenize a single Logseq block string into inline nodes.
///
/// This is intentionally conservative (aims to not mis-parse). Unknown patterns
/// fall back to `Inline::Text`.
///
/// Refactor note: `tokenize_inline` delegates to small recognizers to keep
/// cyclomatic complexity manageable.
pub fn tokenize_inline(input: &str) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::new();
    let mut cur = Cursor::new(input);

    while !cur.is_eof() {
        if let Some(n) = try_code_span(&mut cur) {
            out.push(n);
            continue;
        }
        if let Some(n) = try_embed(&mut cur) {
            out.push(n);
            continue;
        }
        if let Some(n) = try_labeled_link_like(&mut cur) {
            out.push(n);
            continue;
        }
        if let Some(n) = try_page_ref(&mut cur) {
            out.push(n);
            continue;
        }
        if let Some(n) = try_block_ref(&mut cur) {
            out.push(n);
            continue;
        }
        if let Some(n) = try_tag(&mut cur) {
            out.push(n);
            continue;
        }

        // Default: one char of text.
        let ch = cur.take_char();
        push_text_if_needed(&mut out, &ch.to_string());
    }

    coalesce_text(out)
}

#[derive(Debug, Clone)]
struct Cursor<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Cursor<'a> {
    fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }

    fn is_eof(&self) -> bool {
        self.i >= self.s.len()
    }

    fn rest(&self) -> &'a str {
        &self.s[self.i..]
    }

    fn byte(&self) -> u8 {
        self.s.as_bytes()[self.i]
    }

    fn starts_with(&self, pat: &str) -> bool {
        self.rest().starts_with(pat)
    }

    fn advance(&mut self, n: usize) {
        self.i += n;
    }

    fn take_char(&mut self) -> char {
        let ch = self.rest().chars().next().unwrap();
        self.advance(ch.len_utf8());
        ch
    }
}

fn try_code_span(cur: &mut Cursor<'_>) -> Option<Inline> {
    if cur.byte() != b'`' {
        return None;
    }

    let rest = cur.rest();
    let end = rest[1..].find('`')?;
    let code = &rest[1..1 + end];

    cur.advance(1 + end + 1);

    Some(Inline::CodeSpan {
        code: code.to_string(),
    })
}

fn try_embed(cur: &mut Cursor<'_>) -> Option<Inline> {
    if !cur.starts_with("{{embed ") {
        return None;
    }

    let rest = cur.rest();
    let end = rest.find("}}")?;
    let inner = rest["{{embed ".len()..end].trim();
    let target = parse_embed_target(inner)?;

    cur.advance(end + 2);
    Some(Inline::Embed { target })
}

fn try_labeled_link_like(cur: &mut Cursor<'_>) -> Option<Inline> {
    if cur.byte() != b'[' {
        return None;
    }

    let rest = cur.rest();
    let close_bracket = rest[1..].find(']')?;
    let label_raw = &rest[1..1 + close_bracket];

    let after = 1 + close_bracket + 1;
    if after >= rest.len() || rest.as_bytes()[after] != b'(' {
        return None;
    }

    let close_paren = find_matching_paren(rest, after)?;
    let target_raw = &rest[after + 1..close_paren];

    // [label]([[Page]])
    if let Some(page) = parse_page_ref(target_raw) {
        cur.advance(close_paren + 1);
        return Some(Inline::Link {
            label: tokenize_inline(label_raw),
            url: format!("[[{}]]", page.original),
        });
    }

    // [label](((uuid))) (we accept normal block ref patterns too)
    if let Some(uuid) = parse_block_ref(target_raw) {
        cur.advance(close_paren + 1);
        return Some(Inline::Link {
            label: tokenize_inline(label_raw),
            url: format!("(({}))", uuid),
        });
    }

    // standard markdown link [label](url)
    if is_probably_url(target_raw) {
        cur.advance(close_paren + 1);
        return Some(Inline::Link {
            label: tokenize_inline(label_raw),
            url: target_raw.to_string(),
        });
    }

    None
}

fn try_page_ref(cur: &mut Cursor<'_>) -> Option<Inline> {
    if !cur.starts_with("[[") {
        return None;
    }

    let rest = cur.rest();
    let end = rest[2..].find("]]")?;
    let inner = &rest[2..2 + end];

    cur.advance(2 + end + 2);
    Some(Inline::PageRef {
        title: normalize_page_title(inner),
        original: inner.to_string(),
    })
}

fn try_block_ref(cur: &mut Cursor<'_>) -> Option<Inline> {
    if !cur.starts_with("((") {
        return None;
    }

    let rest = cur.rest();
    let end = rest[2..].find("))")?;
    let inner = rest[2..2 + end].trim();

    cur.advance(2 + end + 2);
    Some(Inline::BlockRef {
        uuid: inner.to_string(),
    })
}

#[allow(clippy::manual_strip)]
fn try_tag(cur: &mut Cursor<'_>) -> Option<Inline> {
    if cur.byte() != b'#' {
        return None;
    }

    let rest = cur.rest();

    // #[[multi word]]
    if rest.starts_with("#[[") {
        let end = rest[3..].find("]]")?;
        let inner = &rest[3..3 + end];
        cur.advance(3 + end + 2);
        return Some(Inline::Tag {
            title: normalize_page_title(inner),
            original: inner.to_string(),
        });
    }

    // #word
    let mut j = 1usize;
    while cur.i + j < cur.s.len() {
        let c = cur.s[cur.i + j..].chars().next().unwrap();
        if c.is_whitespace() {
            break;
        }
        if [',', '.', ';', ':', '!', '?', ')', ']', '}', '"', '\''].contains(&c) {
            break;
        }
        j += c.len_utf8();
    }

    if j <= 1 {
        return None;
    }

    let inner = &rest[1..j];
    cur.advance(j);
    Some(Inline::Tag {
        title: normalize_page_title(inner),
        original: inner.to_string(),
    })
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
    // open_idx points at '(' in `s`.
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
    // conservative normalization: trim only
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

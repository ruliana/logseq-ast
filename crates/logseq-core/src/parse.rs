use crate::ast::{Block, Document, Inline, Marker, Property};
use crate::tokenize::tokenize_inline;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("empty input")]
    Empty,

    #[error("unclosed code fence starting at line {line}")]
    UnclosedCodeFence { line: usize },
}

/// Parse a Logseq markdown page into an AST.
///
/// Initial parser rules (v1):
/// - We treat each non-empty line as belonging to a block tree.
/// - Bullets (`- ` or `* `) define explicit blocks; non-bullet text is also
///   accepted as a block at the current indentation level.
/// - Property lines (`key:: value`) attach to the most recent block at the same
///   or deeper indentation level.
pub fn parse(input: &str) -> Result<Document, ParseError> {
    if input.trim().is_empty() {
        return Err(ParseError::Empty);
    }

    let lines: Vec<&str> = input.lines().collect();
    let mut roots: Vec<Block> = Vec::new();
    // stack of indices into the tree, as mutable references are hard;
    // we store a path of (vec index) pointers by walking with helper functions.
    let mut stack: Vec<(usize, usize)> = Vec::new(); // (indent_level, index in current parent's children/roots)

    let mut idx0 = 0usize;
    while idx0 < lines.len() {
        let raw_line = lines[idx0];
        let line_no = idx0 + 1;

        if raw_line.trim().is_empty() {
            idx0 += 1;
            continue;
        }

        let (indent_spaces, trimmed) = split_indent(raw_line);
        let level = indent_spaces / 2;

        // fenced code block? ```lang
        if let Some(info) = parse_code_fence_open(trimmed) {
            let start_line = line_no;
            idx0 += 1;
            let mut body: Vec<&str> = Vec::new();

            while idx0 < lines.len() {
                let l = lines[idx0];
                let (_ind2, t2) = split_indent(l);
                if parse_code_fence_close(t2) {
                    break;
                }
                body.push(l);
                idx0 += 1;
            }

            if idx0 >= lines.len() {
                return Err(ParseError::UnclosedCodeFence { line: start_line });
            }

            // consume closing fence
            idx0 += 1;

            let text = body.join("\n");
            let content = vec![Inline::CodeBlock { info, text }];

            let block = Block {
                id: None,
                marker: None,
                properties: vec![],
                content,
                children: vec![],
                line: start_line,
            };

            place_block(&mut roots, &mut stack, level, block);
            continue;
        }

        // property line?
        if let Some((key, value)) = parse_property_line(trimmed) {
            let prop = Property {
                key: key.to_string(),
                value: value.to_string(),
                line: line_no,
            };

            if let Some(block) = last_block_mut(&mut roots, &stack) {
                // capture id::
                if prop.key == "id" && block.id.is_none() {
                    block.id = Some(prop.value.clone());
                }
                block.properties.push(prop);
            } else {
                // no prior block: create an implicit one
                roots.push(Block {
                    id: if key == "id" {
                        Some(value.to_string())
                    } else {
                        None
                    },
                    marker: None,
                    properties: vec![prop],
                    content: vec![],
                    children: vec![],
                    line: line_no,
                });
                stack.clear();
                stack.push((level, roots.len() - 1));
            }
            idx0 += 1;
            continue;
        }

        let (marker, content_str) = parse_marker(trimmed);
        let content = tokenize_inline(content_str);

        let block = Block {
            id: None,
            marker,
            properties: vec![],
            content,
            children: vec![],
            line: line_no,
        };

        place_block(&mut roots, &mut stack, level, block);
        idx0 += 1;
    }

    Ok(Document {
        version: 1,
        blocks: roots,
    })
}

fn parse_code_fence_open(s: &str) -> Option<Option<String>> {
    let st = s.trim_start();
    if let Some(rest) = st.strip_prefix("```") {
        let info = rest.trim();
        if info.is_empty() {
            return Some(None);
        }
        return Some(Some(info.to_string()));
    }
    None
}

fn parse_code_fence_close(s: &str) -> bool {
    s.trim_start().starts_with("```")
}

fn split_indent(line: &str) -> (usize, &str) {
    let mut spaces = 0usize;
    for c in line.chars() {
        if c == ' ' {
            spaces += 1;
        } else if c == '\t' {
            spaces += 2;
        } else {
            break;
        }
    }
    (spaces, line[spaces.min(line.len())..].trim_start())
}

fn strip_bullet(s: &str) -> (&str, bool) {
    let st = s.trim_start();
    if let Some(rest) = st.strip_prefix("- ") {
        return (rest, true);
    }
    if let Some(rest) = st.strip_prefix("* ") {
        return (rest, true);
    }
    (st, false)
}

fn parse_property_line(s: &str) -> Option<(&str, &str)> {
    let st = s.trim();
    // In Logseq docs: `property:: value`
    let mut it = st.splitn(2, "::");
    let key = it.next()?.trim();
    let val = it.next()?.trim();
    if key.is_empty() {
        return None;
    }
    // avoid false positives for URLs like http:://
    if key.contains(' ') {
        return None;
    }
    Some((key, val))
}

fn parse_marker(s: &str) -> (Option<Marker>, &str) {
    let (without_bullet, _had_bullet) = strip_bullet(s);
    let st = without_bullet.trim_start();

    for (prefix, marker) in [
        ("TODO ", Marker::Todo),
        ("DOING ", Marker::Doing),
        ("DONE ", Marker::Done),
        ("LATER ", Marker::Later),
        ("NOW ", Marker::Now),
    ] {
        if let Some(rest) = st.strip_prefix(prefix) {
            return (Some(marker), rest);
        }
    }

    (None, st)
}

fn place_block(
    roots: &mut Vec<Block>,
    stack: &mut Vec<(usize, usize)>,
    level: usize,
    block: Block,
) {
    // pop until we find parent with indent < level
    while let Some((ind, _)) = stack.last().copied() {
        if ind < level {
            break;
        }
        stack.pop();
    }

    if stack.last().is_some() {
        // insert as child of current top
        let parent = get_block_mut_by_path(roots, stack);
        parent.children.push(block);
        let child_idx = parent.children.len() - 1;
        stack.push((level, child_idx));
    } else {
        roots.push(block);
        let root_idx = roots.len() - 1;
        stack.push((level, root_idx));
    }
}

fn get_block_mut_by_path<'a>(roots: &'a mut [Block], stack: &[(usize, usize)]) -> &'a mut Block {
    // stack[0] indexes roots, stack[n] indexes children of previous
    let mut b: *mut Block = &mut roots[stack[0].1];
    for (_, idx) in stack.iter().skip(1) {
        unsafe {
            // make the intermediate borrow explicit to satisfy linting
            b = &mut (&mut (*b).children)[*idx];
        }
    }
    unsafe { &mut *b }
}

fn last_block_mut<'a>(roots: &'a mut [Block], stack: &[(usize, usize)]) -> Option<&'a mut Block> {
    if stack.is_empty() {
        return None;
    }
    Some(get_block_mut_by_path(roots, stack))
}

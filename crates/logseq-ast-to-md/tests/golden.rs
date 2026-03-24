use std::path::Path;

fn fixture(path: &str) -> String {
    format!("../../fixtures/ast_to_md/{path}")
}

#[test]
fn prose_flattening_basic() {
    let input = std::fs::read_to_string(Path::new(&fixture("prose_basic.json"))).unwrap();
    let doc: logseq_core::ast::Document = serde_json::from_str(&input).unwrap();
    let out = logseq_core::blog::render_blog_markdown(&doc);

    let expected = std::fs::read_to_string(Path::new(&fixture("prose_basic.md"))).unwrap();
    assert_eq!(out, expected);
}

#[test]
fn list_mode_colon_keeps_lists() {
    let input = std::fs::read_to_string(Path::new(&fixture("list_mode.json"))).unwrap();
    let doc: logseq_core::ast::Document = serde_json::from_str(&input).unwrap();
    let out = logseq_core::blog::render_blog_markdown(&doc);

    let expected = std::fs::read_to_string(Path::new(&fixture("list_mode.md"))).unwrap();
    assert_eq!(out, expected);
}

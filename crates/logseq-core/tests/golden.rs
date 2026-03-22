use std::path::Path;

#[test]
fn golden_wiki_and_refs() {
    // Tests run with CWD at the crate, so walk up to workspace root.
    let md = std::fs::read_to_string(Path::new("../../fixtures/wiki_and_refs.md"))
        .expect("read fixture markdown");

    let doc = logseq_core::parse::parse(&md).expect("parse");
    let json = serde_json::to_string_pretty(&doc).expect("serialize");

    // For now, just assert we produce valid JSON and a non-empty document.
    // Once AST schema is finalized, compare against fixtures/wiki_and_refs.json.
    assert!(json.contains("items"));
}

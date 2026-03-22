use std::path::Path;

#[test]
fn golden_wiki_and_refs() {
    // Tests run with CWD at the crate, so walk up to workspace root.
    let md = std::fs::read_to_string(Path::new("../../fixtures/wiki_and_refs.md"))
        .expect("read fixture markdown");

    let doc = logseq_core::parse::parse(&md).expect("parse");
    let json = serde_json::to_string_pretty(&doc).expect("serialize");

    // Golden comparison (behavior-driven): output JSON must match the expected snapshot.
    let expected = std::fs::read_to_string(Path::new("../../fixtures/wiki_and_refs.json"))
        .expect("read expected json");

    assert_eq!(json.trim(), expected.trim());

    // sanity check
    assert!(!doc.blocks.is_empty());
}

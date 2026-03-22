use std::path::Path;

#[test]
fn golden_wiki_and_refs() {
    golden("wiki_and_refs");
}

#[test]
fn golden_code_block() {
    golden("code_block");
}

#[test]
fn golden_code_fence_distinction() {
    golden("code_fence_distinction");
}

#[test]
fn golden_properties_example() {
    golden("properties_example");
}

#[test]
fn golden_property_urls() {
    golden("property_urls");
}

fn golden(name: &str) {
    // Tests run with CWD at the crate, so walk up to workspace root.
    let md = std::fs::read_to_string(Path::new(&format!("../../fixtures/{name}.md")))
        .expect("read fixture markdown");

    let doc = logseq_core::parse::parse(&md).expect("parse");
    let json = serde_json::to_string_pretty(&doc).expect("serialize");

    // Golden comparison (behavior-driven): output JSON must match the expected snapshot.
    let expected = std::fs::read_to_string(Path::new(&format!("../../fixtures/{name}.json")))
        .expect("read expected json");

    assert_eq!(json.trim(), expected.trim());

    // sanity check
    assert!(!doc.blocks.is_empty());
}

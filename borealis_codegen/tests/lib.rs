#![feature(plugin, custom_derive)]
#![plugin(borealis_codegen)]

extern crate borealis;

use borealis::DocumentTemplate;
use borealis::html::Document;

#[derive(DocumentTemplate)]
struct TestTemplate {
    value: String,
}

#[test]
fn test_test_template() {
    let template = TestTemplate {
        value: "Test".to_owned(),
    };
    let document = template.document_template();

    assert_eq!(document, Document::new(None, None));
}

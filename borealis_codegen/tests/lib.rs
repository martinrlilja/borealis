#![feature(plugin, custom_derive)]
#![plugin(borealis_codegen)]

extern crate borealis;

use std::io::Read;
use std::fs::File;
use std::path::Path;

use borealis::DocumentTemplate;
use borealis::html::Document;

#[template_document(file="test_template.html")]
struct TestTemplate {
    value: String,
}

#[test]
fn test_test_template() {
    let template = TestTemplate {
        value: "Test".to_owned(),
    };
    let document_a = template.document_template();
    let document_b = read_document("tests/test_template_expected.html");

    assert_eq!(document_a, document_b);
}

fn read_document<P: AsRef<Path>>(path: P) -> Document {
    let mut file = File::open(path).unwrap();

    let file_str = {
        let mut file_str = String::new();
        file.read_to_string(&mut file_str).unwrap();

        file_str
    };

    Document::parse_str(&file_str)
}
